//! `dog scan` — scan a block range for inscriptions without a full index.
//!
//! Uses direct `.blk` file reads when available (fast), with automatic RPC
//! fallback.  No redb index is required, making this suitable for spot-
//! checking a block range, verifying a specific inscription, or exporting a
//! collection without indexing the entire chain.
//!
//! # Examples
//!
//! Find all inscriptions between two heights:
//! ```text
//! dog scan --from 4609000 --to 4620000
//! ```
//!
//! Verify one inscription by its known txid:
//! ```text
//! dog scan --from 4609000 --to 4700000 --txid bdfeeeacab95d0a230e1124f0635ac9a47925fef4bb1d41a0a0c6e8d8232af7a
//! ```
//!
//! Export all inscriptions owned by an address to disk:
//! ```text
//! dog scan --from 4609000 --to 5000000 --address DHrqn6H... --out ./my-inscriptions
//! ```

use {
  super::*,
  crate::{
    index::updater::blk_reader::BlkReader,
    inscriptions::ParsedEnvelope,
  },
  std::{fs, io::Write, path::PathBuf},
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

impl ScanCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let chain = settings.chain();
    let client = settings.bitcoin_rpc_client(None)?;

    // Open BlkReader for fast disk reads (no index required).
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

    let mut found = 0u64;
    let txid_filter = self.txid.as_deref().map(|s| s.to_lowercase());

    for height in self.from..=self.to {
      // Fetch block: BlkReader first, RPC fallback.
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
        let txid_hex = txid.to_string();

        // txid filter
        if let Some(ref filter) = txid_filter {
          if !txid_hex.starts_with(filter.as_str()) && txid_hex != *filter {
            continue;
          }
        }

        let envelopes = ParsedEnvelope::from_transactions_dogecoin(std::slice::from_ref(tx));
        if envelopes.is_empty() {
          continue;
        }

        // Recipient address: output[0] of the inscription tx.
        let recipient = tx
          .output
          .first()
          .and_then(|o| chain.address_string_from_script(&o.script_pubkey));

        // Address filter.
        if let Some(ref filter) = self.address {
          match &recipient {
            Some(addr) if addr == filter => {}
            _ => continue,
          }
        }

        for (i, envelope) in envelopes.iter().enumerate() {
          let inscription_id = format!("{txid_hex}i{i}");
          let content_type = envelope
            .payload
            .content_type()
            .unwrap_or("unknown")
            .to_string();
          let body_len = envelope.payload.body().map(|b| b.len()).unwrap_or(0);

          found += 1;

          if self.json {
            let obj = serde_json::json!({
              "inscription_id": inscription_id,
              "height": height,
              "txid": txid_hex,
              "index": i,
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

          // Export to filesystem if --out is given.
          if let Some(ref out_dir) = self.out {
            if let Some(body) = envelope.payload.body() {
              let dir_name = format!("{height}_{txid_hex}i{i}");
              let ins_dir = out_dir.join(&dir_name);
              fs::create_dir_all(&ins_dir)?;

              // Content file with appropriate extension.
              let ext = extension_for(&content_type);
              let content_path = ins_dir.join(format!("content{ext}"));
              let mut f = fs::File::create(&content_path)?;
              f.write_all(body)?;

              // Metadata JSON.
              let meta = serde_json::json!({
                "inscription_id": inscription_id,
                "height": height,
                "txid": txid_hex,
                "index": i,
                "content_type": content_type,
                "content_size": body_len,
                "recipient": recipient,
              });
              let meta_path = ins_dir.join("info.json");
              fs::write(meta_path, serde_json::to_string_pretty(&meta)?)?;
            }
          }
        }
      }

      if !self.json && height % 1000 == 0 {
        eprintln!("  scanned through block {height} ({found} inscriptions so far)…");
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

/// Get a block by height via RPC.
fn fetch_block_rpc(client: &bitcoincore_rpc::Client, height: u32) -> crate::Result<bitcoin::Block> {
  use bitcoincore_rpc::RpcApi;
  let hash = client.get_block_hash(height.into())?;
  Ok(client.get_block(&hash)?)
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
