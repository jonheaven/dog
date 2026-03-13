//! Direct `.blk` file reader for Dogecoin Core block data.
//!
//! Reads blocks straight from the binary `.blk` files on disk, bypassing
//! JSON-RPC. Typically 5-20x faster than RPC for initial sync.
//!
//! ## Index copy
//!
//! Dogecoin Core holds an exclusive LevelDB lock on `blocks/index/` while
//! running, which prevents a second process from opening the same DB.
//!
//! To work around this dog maintains a **shadow copy** of the index inside the
//! active Dogecoin Core network data directory at
//! `<dogecoin-data-dir>/<network>/blk-index/`. The copy is refreshed
//! automatically each time `dog index update` runs. A smart-copy strategy is
//! used: immutable SST files (`*.ldb`) are skipped once they already exist in
//! the copy; only the MANIFEST and WAL are re-copied on each run (usually
//! < 1 second). The `LOCK` file is never copied so dog can open the copy
//! freely.
//!
//! The copy can also be refreshed manually:
//! ```text
//! dog index refresh-blk-index
//! ```

use {
  crate::Result,
  anyhow::Context,
  bitcoin::{Block, consensus::deserialize},
  bitcoin::hashes::Hash,
  byteorder::{LittleEndian, ReadBytesExt},
  rusty_leveldb::{DB, LdbIterator, Options},
  std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    io::{BufReader, Cursor, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    time::SystemTime,
  },
};

// Sanity-check: magic constants must be exactly 4 bytes.
const _: () = assert!(crate::chain::chainparams::MAGIC_MAINNET.len() == 4);
const _: () = assert!(crate::chain::chainparams::MAGIC_TESTNET.len() == 4);
const _: () = assert!(crate::chain::chainparams::MAGIC_REGTEST.len() == 4);

/// height → (blk_file_index, data_offset_within_file, block_hash_bytes)
type BlkIndex = HashMap<u32, (u32, u64, [u8; 32])>;

/// Reads blocks directly from Dogecoin Core's `.blk` files.
pub(crate) struct BlkReader {
  blocks_dir: PathBuf,
  index: BlkIndex,
}

impl BlkReader {
  /// Try to open a BlkReader.
  ///
  /// `index_copy_dir` is where dog stores its shadow copy of Core's LevelDB
  /// index (e.g. `<dogecoin-data-dir>/<network>/blk-index`). The copy is
  /// refreshed here before opening so it is always current.
  ///
  /// Fall-through order:
  /// 1. Refresh shadow copy (smart-copy — fast on subsequent runs)
  /// 2. Open shadow copy
  /// 3. Open live index (only succeeds when Core is not running)
  /// 4. Return `Ok(None)` → caller falls back to RPC
  pub(crate) fn open(blocks_dir: &Path, index_copy_dir: &Path) -> Result<Option<Self>> {
    let live_index = blocks_dir.join("index");
    if !live_index.exists() {
      return Ok(None);
    }

    // Refresh the shadow copy.  Safe while Core runs — immutable SST files
    // are skipped if already up-to-date, WAL uses checksums for crash safety.
    match refresh_index_copy(&live_index, index_copy_dir) {
      Ok((copied, skipped)) => {
        if copied > 0 || !index_copy_dir.exists() {
          log::info!(
            "BlkReader: index copy refreshed ({copied} updated, {skipped} unchanged) → {}",
            index_copy_dir.display()
          );
        }
      }
      Err(e) => log::warn!("BlkReader: could not refresh index copy: {e}"),
    }

    // Prefer the shadow copy.
    if index_copy_dir.exists() {
      match build_block_index(index_copy_dir) {
        Ok(idx) => {
          log::info!(
            "BlkReader: loaded {} block locations from shadow copy — using direct .blk reads",
            idx.len()
          );
          return Ok(Some(Self {
            blocks_dir: blocks_dir.to_owned(),
            index: idx,
          }));
        }
        Err(e) => log::warn!("BlkReader: shadow copy unusable ({e}), trying live index"),
      }
    }

    // Fall back to the live index (works when Core is stopped).
    match build_block_index(&live_index) {
      Ok(idx) => {
        log::info!(
          "BlkReader: loaded {} block locations from live index",
          idx.len()
        );
        Ok(Some(Self {
          blocks_dir: blocks_dir.to_owned(),
          index: idx,
        }))
      }
      Err(e) => {
        if e.to_string().contains("lock") {
          log::info!(
            "BlkReader: Dogecoin Core holds the LevelDB lock and the shadow copy could \
             not be created. Falling back to RPC. Run `dog index refresh-blk-index` \
             once to build the shadow copy while Core is running."
          );
        } else {
          log::warn!("BlkReader: could not read block index: {e}");
        }
        Ok(None)
      }
    }
  }

  /// Highest block height available in the on-disk index.
  #[allow(dead_code)]
  pub(crate) fn max_height(&self) -> u32 {
    self.index.keys().copied().max().unwrap_or(0)
  }

  pub(crate) fn location(&self, height: u32) -> Option<(u32, u64, [u8; 32])> {
    self.index.get(&height).copied()
  }

  /// Read and deserialize a block by height.
  ///
  /// Returns `Ok(None)` when the height is not yet indexed (tip blocks that
  /// haven't been flushed to `.blk` files yet — caller falls back to RPC).
  pub(crate) fn get(&self, height: u32) -> Result<Option<Block>> {
    let Some(&(file_idx, data_offset, expected_hash)) = self.index.get(&height) else {
      return Ok(None);
    };
    let block = read_block_from_file(&self.blocks_dir, file_idx, data_offset)
      .with_context(|| format!("reading block at height {height}"))?;

    if block.block_hash().to_byte_array() != expected_hash {
      anyhow::bail!("block hash mismatch for height {height}");
    }

    Ok(Some(block))
  }
}

// ---------------------------------------------------------------------------
// Shadow copy management
// ---------------------------------------------------------------------------

/// Copy Core's `blocks/index/` to `copy_dir`, skipping `LOCK`.
///
/// Uses a **smart-copy** strategy: a file is only copied when the source is
/// newer than the destination (by mtime), so immutable SST files (`*.ldb`)
/// are skipped after the first copy.  The WAL (`*.log`) and MANIFEST change
/// more often and are re-copied on each call.
///
/// Returns `(copied, skipped)` counts.
pub(crate) fn refresh_index_copy(live_index: &Path, copy_dir: &Path) -> Result<(u32, u32)> {
  fs::create_dir_all(copy_dir).with_context(|| format!("creating {}", copy_dir.display()))?;

  let (mut copied, mut skipped) = (0u32, 0u32);

  for entry in
    fs::read_dir(live_index).with_context(|| format!("reading {}", live_index.display()))?
  {
    let entry = entry?;
    let name = entry.file_name();

    // Never copy Core's lock file — that's the whole point of the copy.
    if name == OsStr::new("LOCK") {
      continue;
    }

    let dst = copy_dir.join(&name);

    // Skip if the copy is already at least as new as the source.
    let src_mtime = entry
      .metadata()
      .and_then(|m| m.modified())
      .unwrap_or(SystemTime::UNIX_EPOCH);
    if let Ok(dst_meta) = fs::metadata(&dst) {
      let dst_mtime = dst_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
      if dst_mtime >= src_mtime {
        skipped += 1;
        continue;
      }
    }

    fs::copy(entry.path(), &dst).with_context(|| format!("copying {:?}", name))?;
    copied += 1;
  }

  Ok((copied, skipped))
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

    if let Some((height, file_idx, offset, hash)) = parse_index_record(&key, &value) {
      // first-seen wins; avoids overwriting with stale entries
      result.entry(height).or_insert((file_idx, offset, hash));
    }
  }

  Ok(result)
}

/// Parse a single LevelDB block index value.
///
/// Dogecoin Core record layout:
/// ```text
///   varint  version
///   varint  height
///   varint  status
///   varint  tx_count
///   if status & BLOCK_HAVE_DATA:
///     varint  file_number   (blkNNNNN.dat index)
///     varint  data_offset   (byte offset within that file)
/// ```
fn parse_index_record(key: &[u8], value: &[u8]) -> Option<(u32, u32, u64, [u8; 32])> {
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

  let hash_slice = key.get(1..33)?;
  let mut hash = [0u8; 32];
  hash.copy_from_slice(hash_slice);

  Some((height, file_idx, data_offset, hash))
}

/// Dogecoin Core's LevelDB varint encoding:
/// each byte contributes 7 bits; the high bit signals another byte follows.
fn read_varint(cur: &mut Cursor<&[u8]>) -> std::io::Result<u64> {
  let mut n: u64 = 0;
  loop {
    let byte = cur.read_u8()?;
    n = (n << 7) | (byte & 0x7F) as u64;
    if byte & 0x80 != 0 {
      n = n
        .checked_add(1)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "varint overflow"))?;
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
/// ```text
///   [4 bytes]  network magic  (see chainparams::MAGIC_*)
///   [4 bytes]  block_size     (little-endian u32)
///   [N bytes]  raw serialized block
/// ```
///
/// Magic bytes by network (from [`crate::chain::chainparams`]):
/// - mainnet: `C0 C0 C0 C0`
/// - testnet: `FC C1 B7 DC`
/// - regtest: `FA BF B5 DA`
///
/// The LevelDB `data_offset` points to the start of the raw block bytes
/// (i.e., 8 bytes into the record). So `data_offset - 4` is where
/// `block_size` lives. Because we locate blocks via the LevelDB index
/// rather than linear `.blk` scanning, we never need to read or match the
/// magic bytes at runtime — the constants serve as documentation and for
/// future validation tooling.
pub(crate) fn read_block_from_file(
  blk_dir: &Path,
  file_idx: u32,
  data_offset: u64,
) -> Result<Block> {
  let path = blk_dir.join(format!("blk{:05}.dat", file_idx));
  let file = fs::File::open(&path).with_context(|| format!("opening {}", path.display()))?;
  let mut reader = BufReader::new(file);

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
