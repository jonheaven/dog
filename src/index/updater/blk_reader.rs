//! Direct `.blk` file reader for Dogecoin Core block data.
//!
//! Reads blocks straight from the binary `.blk` files on disk, bypassing
//! JSON-RPC. Typically 5-20x faster than RPC for initial sync.
//!
//! Block location metadata is read from the LevelDB index that Dogecoin Core
//! maintains at `<blocks_dir>/index/`.

use {
  crate::Result,
  anyhow::Context,
  bitcoin::{consensus::deserialize, Block},
  byteorder::{LittleEndian, ReadBytesExt},
  rusty_leveldb::{LdbIterator, Options, DB},
  std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Cursor, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
  },
};

/// height → (blk_file_index, data_offset_within_file)
type BlkIndex = HashMap<u32, (u32, u64)>;

/// Reads blocks directly from Dogecoin Core's `.blk` files.
pub(crate) struct BlkReader {
  blocks_dir: PathBuf,
  index: BlkIndex,
}

impl BlkReader {
  /// Try to open a BlkReader for the given blocks directory.
  ///
  /// Returns `Ok(None)` if the LevelDB index doesn't exist (no Dogecoin Core
  /// data present), `Ok(Some(_))` on success.
  pub(crate) fn open(blocks_dir: &Path) -> Result<Option<Self>> {
    let index_path = blocks_dir.join("index");
    if !index_path.exists() {
      return Ok(None);
    }
    let index =
      build_block_index(&index_path).context("reading LevelDB block index")?;
    log::info!(
      "BlkReader: loaded {} block locations from LevelDB",
      index.len()
    );
    Ok(Some(Self {
      blocks_dir: blocks_dir.to_owned(),
      index,
    }))
  }

  /// Highest block height available in the on-disk index.
  #[allow(dead_code)]
  pub(crate) fn max_height(&self) -> u32 {
    self.index.keys().copied().max().unwrap_or(0)
  }

  /// Read and deserialize a block by height.
  ///
  /// Returns `Ok(None)` when the height is not yet indexed (tip blocks that
  /// haven't been flushed to `.blk` files yet — caller should fall back to RPC).
  pub(crate) fn get(&self, height: u32) -> Result<Option<Block>> {
    let Some(&(file_idx, data_offset)) = self.index.get(&height) else {
      return Ok(None);
    };
    let block = read_block_from_file(&self.blocks_dir, file_idx, data_offset)
      .with_context(|| format!("reading block at height {height}"))?;
    Ok(Some(block))
  }
}

// ---------------------------------------------------------------------------
// LevelDB block index parsing
// ---------------------------------------------------------------------------

fn build_block_index(index_path: &Path) -> Result<BlkIndex> {
  let mut opts = Options::default();
  opts.create_if_missing = false;

  let mut db = DB::open(index_path, opts)
    .with_context(|| format!("opening LevelDB at {}", index_path.display()))?;

  let mut result = BlkIndex::new();
  let mut iter = db.new_iter()?;

  // Bitcoin/Dogecoin Core block records all start with key prefix b'b'
  iter.seek(b"b");

  let (mut key, mut value) = (vec![], vec![]);
  while iter.advance() {
    iter.current(&mut key, &mut value);

    if key.first() != Some(&b'b') {
      break; // past the block records
    }

    if let Some((height, file_idx, offset)) = parse_index_record(&value) {
      // first-seen wins; avoids overwriting with stale entries
      result.entry(height).or_insert((file_idx, offset));
    }
  }

  Ok(result)
}

/// Parse a single LevelDB block index value.
///
/// Bitcoin Core record layout:
/// ```
///   varint  version
///   varint  height
///   varint  status
///   varint  tx_count
///   if status & BLOCK_HAVE_DATA:
///     varint  file_number   (blkNNNNN.dat index)
///     varint  data_offset   (byte offset within that file)
/// ```
fn parse_index_record(value: &[u8]) -> Option<(u32, u32, u64)> {
  let mut cur = Cursor::new(value);

  let _version = read_varint(&mut cur).ok()?;
  let height = read_varint(&mut cur).ok()? as u32;
  let status = read_varint(&mut cur).ok()?;
  let _tx_count = read_varint(&mut cur).ok()?;

  const BLOCK_HAVE_DATA: u64 = 8;
  const BLOCK_FAILED_VALID: u64 = 32;
  const BLOCK_FAILED_CHILD: u64 = 64;

  if status & BLOCK_HAVE_DATA == 0 {
    return None; // data not on disk
  }
  if status & (BLOCK_FAILED_VALID | BLOCK_FAILED_CHILD) != 0 {
    return None; // invalid block, skip
  }

  let file_idx = read_varint(&mut cur).ok()? as u32;
  let data_offset = read_varint(&mut cur).ok()?;

  Some((height, file_idx, data_offset))
}

/// Bitcoin Core's LevelDB varint encoding:
/// each byte contributes 7 bits; the high bit signals another byte follows.
/// After reading, increment n for each continuation byte (n += 1 per MSB set).
fn read_varint(cur: &mut Cursor<&[u8]>) -> std::io::Result<u64> {
  let mut n: u64 = 0;
  loop {
    let byte = cur.read_u8()?;
    n = (n << 7) | (byte & 0x7F) as u64;
    if byte & 0x80 != 0 {
      n = n.checked_add(1).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "varint overflow")
      })?;
    } else {
      break;
    }
  }
  Ok(n)
}

// ---------------------------------------------------------------------------
// .blk file binary reading
// ---------------------------------------------------------------------------

/// Each block record in a `.blk` file:
/// ```
///   [4 bytes]  network magic  (0xC0C0C0C0 for Dogecoin mainnet)
///   [4 bytes]  block_size     (little-endian u32)
///   [N bytes]  raw serialized block
/// ```
///
/// The LevelDB `data_offset` points to the start of the raw block bytes
/// (i.e., 8 bytes into the record). So `data_offset - 4` is where
/// `block_size` lives.
fn read_block_from_file(blk_dir: &Path, file_idx: u32, data_offset: u64) -> Result<Block> {
  let path = blk_dir.join(format!("blk{:05}.dat", file_idx));
  let file =
    File::open(&path).with_context(|| format!("opening {}", path.display()))?;
  let mut reader = BufReader::new(file);

  // data_offset - 4 is the block_size field
  reader
    .seek(SeekFrom::Start(data_offset.saturating_sub(4)))
    .context("seeking to block_size")?;

  let block_size = reader
    .read_u32::<LittleEndian>()
    .context("reading block_size")? as usize;

  let mut block_bytes = vec![0u8; block_size];
  reader
    .read_exact(&mut block_bytes)
    .context("reading block bytes")?;

  deserialize(&block_bytes).context("deserializing block")
}
