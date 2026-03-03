//! `dog scan` — scan a block range for inscriptions without a full index.
//!
//! Two-pass scan:
//! 1. Fetch every block in the range; store per-transaction push data and the
//!    UTXO spend map.
//! 2. Emit single-part inscriptions (standard envelope) and reassemble
//!    multi-part inscriptions (Doginals chunked multi-tx format).
//!
//! # Multi-part format (apezord-compatible)
//!
//! Main tx:
//!   `ord | N | content_type | [idx↓, 240-byte chunk]* | sig | pubkey`
//!
//! Continuation txs (each spends output[0] of the previous tx in the chain):
//!   `[idx↓, 240-byte chunk]* | sig | pubkey`
//!
//! Chunk indices count DOWN from `N-1` to `0`.  Sorting by descending index
//! gives the bytes in the correct order.  The spending chain links each
//! continuation tx back to the original inscription without ambiguity.

use {
  super::*,
  crate::{
    index::updater::blk_reader::BlkReader,
  },
  std::{collections::HashMap, collections::HashSet, fs, io::Write, path::PathBuf},
};

#[derive(Clone, Debug, Parser)]
pub struct ScanCommand {
  #[arg(long, help = "Start block height (inclusive)")]
  pub from: u32,

  #[arg(long, help = "End block height (inclusive)")]
  pub to: u32,

  #[arg(long, help = "Only show inscriptions sent to this address")]
  pub address: Option<String>,

  #[arg(
    long,
    help = "Only show the inscription with this txid (finds exact inscription ID)"
  )]
  pub txid: Option<String>,

  #[arg(long, help = "Write content + metadata for each inscription to this directory")]
  pub out: Option<PathBuf>,

  #[arg(long, help = "Output as JSON")]
  pub json: bool,
}

/// Simplified push item extracted from a scriptSig.
#[derive(Debug, Clone)]
enum SPush {
  Ord,             // b"ord" — the Doginals protocol marker
  Int(u16),        // 2-byte little-endian integer (piece count or chunk index)
  Bytes(Vec<u8>),  // any other data push (tag, value, body chunk, sig, pubkey…)
}

/// Per-transaction data collected during the first pass.
struct TxRecord {
  height: u32,
  pushes: Vec<SPush>,
  /// input[0].previous_output.txid — None for coinbase transactions.
  spends: Option<bitcoin::Txid>,
  recipient: Option<String>,
}

impl ScanCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let chain = settings.chain();
    let client = settings.dogecoin_rpc_client(None)?;

    let index_copy_dir = settings.data_dir().join("blk-index");
    let reader = settings
      .dogecoin_blocks_dir()
      .and_then(|dir| BlkReader::open(&dir, &index_copy_dir).ok().flatten());

    if reader.is_none() {
      log::info!("BlkReader unavailable — using RPC for block fetching (slower)");
    }

    if let Some(ref out) = self.out {
      fs::create_dir_all(out)?;
    }

    // ── Pass 1: fetch all blocks, build transaction records and spend map ─────

    let mut all_txs: HashMap<bitcoin::Txid, TxRecord> = HashMap::new();
    // spent_by[parent_txid] = txid_that_spent_it
    let mut spent_by: HashMap<bitcoin::Txid, bitcoin::Txid> = HashMap::new();
    // Block-order txid list so we emit inscriptions chronologically.
    let mut ordered_txids: Vec<bitcoin::Txid> = Vec::new();

    for height in self.from..=self.to {
      let block = if let Some(ref r) = reader {
        match r.get(height)? {
          Some(b) => b,
          None => fetch_block_rpc(&client, height)?,
        }
      } else {
        fetch_block_rpc(&client, height)?
      };

      for tx in &block.txdata {
        let txid = tx.compute_txid();
        let pushes = parse_scriptSig_pushes(tx);

        let spends = if tx.input[0].previous_output.is_null() {
          None
        } else {
          let parent = tx.input[0].previous_output.txid;
          spent_by.insert(parent, txid);
          Some(parent)
        };

        let recipient = tx
          .output
          .first()
          .and_then(|o| chain.address_string_from_script(&o.script_pubkey));

        ordered_txids.push(txid);
        all_txs.insert(txid, TxRecord { height, pushes, spends, recipient });
      }

      if !self.json && height % 1000 == 0 {
        eprintln!("  fetched block {height}...");
      }
    }

    // ── Pass 2: detect and emit inscriptions ──────────────────────────────────

    let txid_filter = self.txid.as_deref().map(|s| s.to_lowercase());
    let mut found = 0u64;
    // Multi-part main txids — skip these in the single-part pass.
    let mut multipart_txids: HashSet<bitcoin::Txid> = HashSet::new();

    // 2a. Multi-part inscriptions (ord + Int(N) header, followed by spending chain).
    for &txid in &ordered_txids {
      let record = &all_txs[&txid];
      let (piece_count, content_type, mut chunks) =
        match parse_multipart_header(&record.pushes) {
          Some(h) => h,
          None => continue,
        };

      multipart_txids.insert(txid);

      // Follow the UTXO spending chain to collect continuation chunks.
      let mut cur = txid;
      while chunks.len() < piece_count as usize {
        match spent_by.get(&cur) {
          Some(&next) => match all_txs.get(&next) {
            Some(next_rec) => {
              collect_continuation_chunks(&next_rec.pushes, &mut chunks);
              cur = next;
            }
            None => {
              log::warn!(
                "multi-part {txid}: continuation {next} is outside the scan range — \
                 increase --to or re-run with a wider range"
              );
              break;
            }
          },
          None => break,
        }
      }

      let txid_hex = txid.to_string();
      if !passes_filters(&txid_hex, &record.recipient, &txid_filter, &self.address) {
        continue;
      }

      // Sort chunks by index descending: highest index = first byte of content.
      let body = reassemble_chunks(&chunks);
      let inscription_id = format!("{txid_hex}i0");

      found += 1;
      emit_inscription(
        &self,
        &inscription_id,
        &txid_hex,
        0,
        record.height,
        &content_type,
        &body,
        &record.recipient,
      )?;
    }

    // 2b. Single-part inscriptions (standard ord envelope).
    for &txid in &ordered_txids {
      if multipart_txids.contains(&txid) {
        continue;
      }

      let record = &all_txs[&txid];
      let txid_hex = txid.to_string();

      if !passes_filters(&txid_hex, &record.recipient, &txid_filter, &self.address) {
        continue;
      }

      if let Some((content_type, body)) = parse_singlepart(&record.pushes) {
        let inscription_id = format!("{txid_hex}i0");
        found += 1;
        emit_inscription(
          &self,
          &inscription_id,
          &txid_hex,
          0,
          record.height,
          &content_type,
          &body,
          &record.recipient,
        )?;
      }
    }

    if !self.json {
      println!(
        "\nScanned blocks {}–{}: {} inscription{} found.",
        self.from,
        self.to,
        found,
        if found == 1 { "" } else { "s" },
      );
      if let Some(ref out) = self.out {
        println!("Content written to: {}", out.display());
      }
    }

    Ok(None)
  }
}

// ── Push parsing ──────────────────────────────────────────────────────────────

/// Parse `input[0].script_sig` into a flat list of simplified push items.
fn parse_scriptSig_pushes(tx: &bitcoin::Transaction) -> Vec<SPush> {
  let script = match tx.input.first() {
    Some(i) => i.script_sig.as_bytes(),
    None => return Vec::new(),
  };

  let mut result = Vec::new();
  let mut i = 0usize;

  while i < script.len() {
    let op = script[i];
    i += 1;

    // OP_1 through OP_16 (0x51–0x60): small integer encoded directly in the
    // opcode byte — no following data bytes.  Used for remaining_count 1-16
    // in multi-part inscriptions (H3imdall-dev encoding).
    if (0x51..=0x60).contains(&op) {
      result.push(SPush::Bytes(vec![op - 0x50]));
      continue;
    }

    let chunk: &[u8] = if op == 0x00 {
      // OP_0 — pushes empty bytes (body separator in the envelope format).
      &[]
    } else if (0x01..=0x4b).contains(&op) {
      // Direct pushdata: next `op` bytes are the data.
      let end = (i + op as usize).min(script.len());
      let c = &script[i..end];
      i = end;
      c
    } else if op == 0x4c {
      // OP_PUSHDATA1: 1-byte length prefix.
      if i >= script.len() {
        break;
      }
      let n = script[i] as usize;
      i += 1;
      let end = (i + n).min(script.len());
      let c = &script[i..end];
      i = end;
      c
    } else if op == 0x4d {
      // OP_PUSHDATA2: 2-byte LE length prefix.
      if i + 2 > script.len() {
        break;
      }
      let n = u16::from_le_bytes([script[i], script[i + 1]]) as usize;
      i += 2;
      let end = (i + n).min(script.len());
      let c = &script[i..end];
      i = end;
      c
    } else {
      // Non-push opcode — not an inscription scriptSig, stop.
      break;
    };

    if chunk == b"ord" {
      result.push(SPush::Ord);
    } else if chunk.len() == 2 {
      result.push(SPush::Int(u16::from_le_bytes([chunk[0], chunk[1]])));
    } else {
      result.push(SPush::Bytes(chunk.to_vec()));
    }
  }

  result
}

// ── Multi-part detection and reassembly ───────────────────────────────────────

/// Return `(piece_count, content_type, initial_chunks)` if `pushes` is the
/// header of a multi-part inscription, otherwise `None`.
fn parse_multipart_header(pushes: &[SPush]) -> Option<(u16, String, HashMap<u16, Vec<u8>>)> {
  // Must start: Ord, Int(N > 1), Bytes(content_type)
  if pushes.len() < 3 {
    return None;
  }
  matches!(pushes[0], SPush::Ord).then_some(())?;
  let piece_count = match pushes[1] {
    SPush::Int(n) if n > 1 => n,
    _ => return None,
  };
  let content_type = match &pushes[2] {
    SPush::Bytes(b) if !b.is_empty() => String::from_utf8(b.clone()).ok()?,
    _ => return None,
  };

  // Collect initial (idx, data) pairs from pushes[3..] minus the last 2
  // (signature and pubkey at the tail of every P2PKH scriptSig).
  let mut chunks = HashMap::new();
  let data_end = if pushes.len() >= 2 { pushes.len() - 2 } else { 0 };
  collect_int_data_pairs(&pushes[3..data_end], &mut chunks);

  Some((piece_count, content_type, chunks))
}

/// Extract continuation chunks from a continuation tx's pushes.
/// Strip the last 2 pushes (signature + pubkey).
fn collect_continuation_chunks(pushes: &[SPush], chunks: &mut HashMap<u16, Vec<u8>>) {
  let data_end = if pushes.len() >= 2 { pushes.len() - 2 } else { 0 };
  collect_int_data_pairs(&pushes[..data_end], chunks);
}

/// Walk a push slice and insert every `(idx, data)` pair into `chunks`.
///
/// Handles all three remaining_count encodings used by Doginals tools:
/// - `SPush::Int(n)`          — 2-byte LE pushdata  → remaining_count 128-N
/// - `SPush::Bytes(b)` b==[]  — OP_0               → remaining_count 0 (last chunk)
/// - `SPush::Bytes(b)` b.len()==1 — 1-byte pushdata → remaining_count 1-127
///
/// The `data.len() >= 20` guard prevents false positives (e.g. confusing a
/// 1-byte tag field with a chunk index).
fn collect_int_data_pairs(pushes: &[SPush], chunks: &mut HashMap<u16, Vec<u8>>) {
  let mut i = 0;
  while i + 1 < pushes.len() {
    let idx_opt: Option<u16> = match &pushes[i] {
      SPush::Int(n)              => Some(*n),
      SPush::Bytes(b) if b.is_empty() => Some(0),
      SPush::Bytes(b) if b.len() == 1 => Some(b[0] as u16),
      _ => None,
    };
    if let Some(idx) = idx_opt {
      if let SPush::Bytes(data) = &pushes[i + 1] {
        if data.len() >= 20 {
          chunks.insert(idx, data.clone());
          i += 2;
          continue;
        }
      }
    }
    i += 1;
  }
}

/// Sort chunks by index descending and concatenate into the final byte stream.
/// Descending order = natural content order (highest index = first bytes).
fn reassemble_chunks(chunks: &HashMap<u16, Vec<u8>>) -> Vec<u8> {
  let mut pairs: Vec<(u16, &Vec<u8>)> = chunks.iter().map(|(k, v)| (*k, v)).collect();
  pairs.sort_by(|a, b| b.0.cmp(&a.0));
  pairs.iter().flat_map(|(_, d)| d.iter().copied()).collect()
}

// ── Single-part inscription parsing ──────────────────────────────────────────

/// Parse a standard single-part Doginals envelope from stored push data.
/// Returns `(content_type, body)` or `None` if no valid inscription found.
fn parse_singlepart(pushes: &[SPush]) -> Option<(String, Vec<u8>)> {
  // Must start with Ord, and second push must NOT be Int (that's multi-part).
  if pushes.is_empty() || !matches!(pushes[0], SPush::Ord) {
    return None;
  }
  if matches!(pushes.get(1), Some(SPush::Int(_))) {
    return None;
  }

  let mut content_type = String::new();
  let mut body: Vec<u8> = Vec::new();
  let mut in_body = false;
  let mut i = 1usize;

  while i < pushes.len() {
    if in_body {
      if let SPush::Bytes(chunk) = &pushes[i] {
        body.extend_from_slice(chunk);
      }
      i += 1;
      continue;
    }

    match &pushes[i] {
      // Empty push at a tag position = body separator.
      SPush::Bytes(b) if b.is_empty() => {
        in_body = true;
        i += 1;
      }
      // 1-byte tag.
      SPush::Bytes(b) if b.len() == 1 => {
        let tag = b[0];
        let val = pushes.get(i + 1);
        if tag == 0x01 {
          // content_type
          if let Some(SPush::Bytes(v)) = val {
            content_type = String::from_utf8_lossy(v).to_string();
          }
        }
        // Skip tag + value regardless (handles unknown tags gracefully).
        i += 2;
      }
      _ => {
        i += 1;
      }
    }
  }

  if content_type.is_empty() && body.is_empty() {
    return None;
  }

  let ct = if content_type.is_empty() {
    "application/octet-stream".to_string()
  } else {
    content_type
  };

  Some((ct, body))
}

// ── Output helpers ────────────────────────────────────────────────────────────

fn passes_filters(
  txid_hex: &str,
  recipient: &Option<String>,
  txid_filter: &Option<String>,
  address_filter: &Option<String>,
) -> bool {
  if let Some(f) = txid_filter {
    if !txid_hex.starts_with(f.as_str()) && txid_hex != f {
      return false;
    }
  }
  if let Some(f) = address_filter {
    match recipient {
      Some(addr) if addr == f => {}
      _ => return false,
    }
  }
  true
}

#[allow(clippy::too_many_arguments)]
fn emit_inscription(
  cmd: &ScanCommand,
  inscription_id: &str,
  txid_hex: &str,
  index: usize,
  height: u32,
  content_type: &str,
  body: &[u8],
  recipient: &Option<String>,
) -> crate::Result<()> {
  let body_len = body.len();

  if cmd.json {
    let obj = serde_json::json!({
      "inscription_id": inscription_id,
      "height": height,
      "txid": txid_hex,
      "index": index,
      "content_type": content_type,
      "content_size": body_len,
      "recipient": recipient,
    });
    println!("{}", serde_json::to_string_pretty(&obj)?);
  } else {
    println!(
      "[{}] height={} type={} size={} bytes  recipient={}",
      inscription_id,
      height,
      content_type,
      body_len,
      recipient.as_deref().unwrap_or("?"),
    );
  }

  if let Some(ref out_dir) = cmd.out {
    if !body.is_empty() {
      let dir_name = format!("{height}_{inscription_id}");
      let ins_dir = out_dir.join(&dir_name);
      fs::create_dir_all(&ins_dir)?;

      let ext = extension_for(content_type);
      let mut f = fs::File::create(ins_dir.join(format!("content{ext}")))?;
      f.write_all(body)?;

      let meta = serde_json::json!({
        "inscription_id": inscription_id,
        "height": height,
        "txid": txid_hex,
        "index": index,
        "content_type": content_type,
        "content_size": body_len,
        "recipient": recipient,
      });
      fs::write(ins_dir.join("info.json"), serde_json::to_string_pretty(&meta)?)?;
    }
  }

  Ok(())
}

// ── Block fetching ────────────────────────────────────────────────────────────

/// Get a block by height via RPC.
///
/// Dogecoin blocks after height 371,337 carry an AuxPoW header that the
/// `bitcoin` crate cannot deserialize. We work around this by fetching the
/// list of txids first (`getblock … 1`), then pulling each transaction
/// individually via `getrawtransaction`. Transactions are plain legacy format
/// (no AuxPoW), so they deserialize fine.
fn fetch_block_rpc(client: &bitcoincore_rpc::Client, height: u32) -> crate::Result<bitcoin::Block> {
  use {
    bitcoin::consensus::deserialize,
    bitcoincore_rpc::RpcApi,
    serde::Deserialize,
  };

  let hash = client.get_block_hash(height.into())?;

  // Only decode the txid list — avoids AuxPoW deserialization of the header.
  #[derive(Deserialize)]
  struct BlockTxids {
    tx: Vec<bitcoin::Txid>,
  }
  let info: BlockTxids =
    client.call("getblock", &[serde_json::to_value(hash)?, serde_json::Value::from(1u8)])?;

  // Fetch and deserialize each transaction individually.
  let txdata = info
    .tx
    .iter()
    .map(|txid| {
      let hex: String =
        client.call("getrawtransaction", &[serde_json::to_value(txid)?, serde_json::Value::from(false)])?;
      let bytes = hex::decode(hex.trim())?;
      deserialize::<bitcoin::Transaction>(&bytes).map_err(|e| anyhow::anyhow!(e))
    })
    .collect::<crate::Result<Vec<_>>>()?;

  // Construct a minimal Block — scan only uses `txdata`.
  Ok(bitcoin::Block {
    header: bitcoin::block::Header {
      version: bitcoin::block::Version::ONE,
      prev_blockhash: bitcoin::BlockHash::all_zeros(),
      merkle_root: bitcoin::TxMerkleNode::all_zeros(),
      time: 0,
      bits: bitcoin::CompactTarget::from_consensus(0),
      nonce: 0,
    },
    txdata,
  })
}

/// Map a MIME content-type to a file extension for the exported content file.
fn extension_for(content_type: &str) -> &'static str {
  let base = content_type.split(';').next().unwrap_or("").trim();
  match base {
    "image/png" => ".png",
    "image/jpeg" | "image/jpg" => ".jpg",
    "image/gif" => ".gif",
    "image/webp" => ".webp",
    "image/svg+xml" => ".svg",
    "image/avif" => ".avif",
    "text/plain" => ".txt",
    "text/html" => ".html",
    "application/json" => ".json",
    "audio/mpeg" | "audio/mp3" => ".mp3",
    "audio/wav" | "audio/wave" => ".wav",
    "video/mp4" => ".mp4",
    "video/webm" => ".webm",
    "model/gltf+json" | "model/gltf-binary" => ".gltf",
    _ => ".bin",
  }
}
