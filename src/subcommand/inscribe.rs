//! `dog inscribe` — write a Dogecoin inscription to the chain.
//!
//! Uses the H3imdall-dev / apezord multi-part format compatible with all major
//! Doginals tooling.  The scriptSig of input[0] carries the inscription data:
//!
//! **Main tx:**
//! ```text
//! PUSH("ord") | PUSH(N:u16LE) | PUSH(mime) | [PUSH(count↓) PUSH(chunk)]* | PUSH(sig) | PUSH(pubkey)
//! ```
//!
//! **Continuation txs** (each spends output[0] of the previous tx):
//! ```text
//! [PUSH(count↓) PUSH(chunk)]* | PUSH(sig) | PUSH(pubkey)
//! ```
//!
//! `count` starts at N-1 and decrements to 0 (0 = last chunk).
//!
//! # Signing strategy
//!
//! P2PKH sighash covers the *scriptCode* (the prevout's scriptPubKey), not the
//! scriptSig.  A signature produced for a template tx with an empty scriptSig is
//! therefore valid for the same tx with inscription data prepended — the script
//! engine consumes only the top two stack items (sig + pubkey) via OP_CHECKSIG.
//!
//! Flow per tx:
//!   1. Build a signing template with an empty scriptSig.
//!   2. Call `signrawtransactionwithwallet` — Core fills in `<sig> <pubkey>`.
//!   3. Extract the raw sig and pubkey bytes.
//!   4. Build the actual scriptSig: inscription segments + sig + pubkey.
//!   5. Broadcast with `sendrawtransaction`.
//!
//! No private keys are ever exported from Core.

use {
  super::*,
  bitcoin::{
    Amount, OutPoint, Sequence, Transaction, TxIn, TxOut, Txid,
    consensus::serialize,
  },
  bitcoincore_rpc::{RpcApi, json::SignRawTransactionInput},
  std::{fs, path::{Path, PathBuf}},
};

/// Maximum bytes of inscription content per chunk push.
const CHUNK_SIZE: usize = 240;

/// Maximum total chunk-data bytes to embed per transaction (~6 chunks).
const MAX_PAYLOAD: usize = 1500;

/// Default postage attached to the inscription output (koinu).
const POSTAGE_DEFAULT: u64 = 100_000;

#[derive(Debug, Parser)]
pub struct InscribeCommand {
  #[arg(long, help = "File to inscribe")]
  pub file: PathBuf,

  #[arg(
    long,
    default_value = "1.0",
    help = "Fee rate in koinu per virtual byte"
  )]
  pub fee_rate: f64,

  #[arg(
    long,
    help = "Send the inscription to this Dogecoin address (default: fresh wallet address)"
  )]
  pub destination: Option<String>,

  #[arg(long, help = "Dogecoin Core wallet name (default: uses the default wallet)")]
  pub wallet: Option<String>,

  #[arg(long, help = "Print transaction details without broadcasting anything")]
  pub dry_run: bool,

  #[arg(
    long,
    help = "Koinu to attach to the inscription output (default: 100000 = 0.001 DOGE)"
  )]
  pub postage: Option<u64>,
}

impl InscribeCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let postage = self.postage.unwrap_or(POSTAGE_DEFAULT);
    let client = settings.dogecoin_rpc_client(self.wallet.clone())?;

    // ── Read file, detect MIME, split into 240-byte chunks ───────────────────

    let data = fs::read(&self.file)?;
    let mime = detect_mime(&self.file, &data);
    let all_chunks: Vec<&[u8]> = data.chunks(CHUNK_SIZE).collect();
    let n_chunks = all_chunks.len();

    // Group chunks into transactions (≤ MAX_PAYLOAD bytes of data each).
    let chunks_per_tx = MAX_PAYLOAD / CHUNK_SIZE; // 6
    let tx_groups: Vec<Vec<&[u8]>> = all_chunks
      .chunks(chunks_per_tx)
      .map(|g| g.to_vec())
      .collect();
    let n_txs = tx_groups.len();
    let n_txs_u16 = u16::try_from(n_txs)
      .map_err(|_| anyhow!("file too large: would require more than 65535 transactions"))?;

    eprintln!(
      "Inscribing {} ({} bytes, {}) → {} chunk{} across {} transaction{}.",
      self.file.display(),
      data.len(),
      mime,
      n_chunks,
      if n_chunks == 1 { "" } else { "s" },
      n_txs,
      if n_txs == 1 { "" } else { "s" },
    );

    // ── Build pre-encoded script segments for each tx ─────────────────────────
    //
    // Each element is a `Vec<u8>` of already-encoded script bytes (opcodes +
    // data), NOT raw data.  `build_script_sig` concatenates them and appends
    // the sig+pubkey pushes.

    let mut segments_per_tx: Vec<Vec<Vec<u8>>> = Vec::with_capacity(n_txs);
    let mut global_chunk_idx: u16 = 0;

    for (tx_idx, group) in tx_groups.iter().enumerate() {
      let mut segs: Vec<Vec<u8>> = Vec::new();

      if tx_idx == 0 {
        // Main tx header: ord marker, piece count, content type.
        segs.push(encode_push(b"ord"));
        segs.push(encode_push(&n_txs_u16.to_le_bytes())); // always 2-byte LE push
        segs.push(encode_push(mime.as_bytes()));
      }

      for chunk in group.iter() {
        // remaining_count counts down from (n_chunks-1) to 0.
        let remaining = (n_chunks as u16 - 1) - global_chunk_idx;
        segs.push(encode_count(remaining)); // uses OP_0 / OP_n / pushdata
        segs.push(encode_push(chunk));
        global_chunk_idx += 1;
      }

      segments_per_tx.push(segs);
    }

    // ── Compute per-tx fees and carry-forward values ───────────────────────────
    //
    // carry[i] = value of output[0] of tx[i].
    //   carry[last]    = postage (recipient gets this)
    //   carry[i]       = carry[i+1] + fees[i+1]  (enough to fund all remaining txs)
    //
    // TX0 is the only tx with a large UTXO as input, so only TX0 has a change
    // output (2 outputs total).  Continuation txs have 1 output each.

    let fees: Vec<u64> = (0..n_txs)
      .map(|i| {
        let n_out = if i == 0 { 2 } else { 1 };
        calc_fee(script_sig_size(&segments_per_tx[i]), n_out, self.fee_rate)
      })
      .collect();

    let mut carry = vec![0u64; n_txs];
    carry[n_txs - 1] = postage;
    for i in (0..n_txs - 1).rev() {
      carry[i] = carry[i + 1] + fees[i + 1];
    }

    let total_fees: u64 = fees.iter().sum();
    let total_needed: u64 = carry[0] + fees[0]; // = postage + total_fees
    eprintln!(
      "Estimated total fees: {} koinu ({:.4} DOGE).  Total needed from wallet: {} koinu.",
      total_fees,
      total_fees as f64 / 1e8,
      total_needed,
    );

    // ── Select funding UTXO ───────────────────────────────────────────────────

    let (utxo_txid, utxo_vout, utxo_value, utxo_script) =
      select_utxo(&client, total_needed)?;

    let change_amount = utxo_value.to_sat().saturating_sub(total_needed);

    // ── Get output scripts ────────────────────────────────────────────────────

    // Change goes back to a fresh wallet address (only needed when change > 0).
    let change_script: Option<bitcoin::ScriptBuf> = if change_amount > 0 {
      let addr: String = client.call("getrawchangeaddress", &[])?;
      Some(parse_dogecoin_address(&addr)?)
    } else {
      None
    };

    // Inscription recipient.
    let recipient_script = match &self.destination {
      Some(addr) => parse_dogecoin_address(addr)?,
      None => {
        let addr: String = client.call("getnewaddress", &[])?;
        parse_dogecoin_address(&addr)?
      }
    };

    // The sender's scriptPubKey — used for all carry-forward outputs so the
    // same wallet key can sign each continuation tx.
    let sender_script = utxo_script.clone();

    // ── Build, sign, and broadcast the transaction chain ─────────────────────

    let mut inscription_txid: Option<Txid> = None;
    let mut prev_txid = utxo_txid;
    let mut prev_vout = utxo_vout;
    let mut prev_value = utxo_value;
    let mut prev_script = utxo_script;

    for tx_idx in 0..n_txs {
      let is_last = tx_idx == n_txs - 1;
      let is_first = tx_idx == 0;

      // Build outputs.
      let mut outputs: Vec<TxOut> = Vec::new();

      // Output[0]: inscription receiver (recipient) if last tx, else carry-forward.
      let out0_script = if is_last {
        recipient_script.clone()
      } else {
        sender_script.clone()
      };
      let out0_value = if is_last { postage } else { carry[tx_idx] };
      outputs.push(TxOut {
        value: Amount::from_sat(out0_value),
        script_pubkey: out0_script,
      });

      // Output[1]: change — only TX0 has a large UTXO input with change.
      if is_first {
        if let Some(ref cscript) = change_script {
          outputs.push(TxOut {
            value: Amount::from_sat(change_amount),
            script_pubkey: cscript.clone(),
          });
        }
      }

      // Build the signing template (empty scriptSig).
      let template = Transaction {
        version: bitcoin::transaction::Version(1),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
          previous_output: OutPoint {
            txid: prev_txid,
            vout: prev_vout,
          },
          script_sig: bitcoin::ScriptBuf::new(),
          sequence: Sequence::MAX,
          witness: bitcoin::Witness::new(),
        }],
        output: outputs.clone(),
      };

      // Sign via Core → extract sig + pubkey bytes.
      let (sig_bytes, pubkey_bytes) =
        sign_template(&client, &template, prev_txid, prev_vout, &prev_script, prev_value)?;

      // Assemble the actual scriptSig: inscription data + sig + pubkey.
      let raw_script_sig =
        build_script_sig(&segments_per_tx[tx_idx], &sig_bytes, &pubkey_bytes);
      let actual_script_sig = bitcoin::ScriptBuf::from(raw_script_sig);

      // Build the final transaction.
      let final_tx = Transaction {
        version: bitcoin::transaction::Version(1),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
          previous_output: OutPoint {
            txid: prev_txid,
            vout: prev_vout,
          },
          script_sig: actual_script_sig,
          sequence: Sequence::MAX,
          witness: bitcoin::Witness::new(),
        }],
        output: outputs,
      };

      let final_txid = final_tx.compute_txid();

      if self.dry_run {
        eprintln!(
          "  TX {}/{} txid={final_txid} (dry-run, not broadcast)",
          tx_idx + 1,
          n_txs
        );
      } else {
        let hex = hex::encode(serialize(&final_tx));
        let _: serde_json::Value =
          client.call("sendrawtransaction", &[serde_json::Value::String(hex)])?;
        eprintln!("  TX {}/{} txid={final_txid}", tx_idx + 1, n_txs);
      }

      if is_first {
        inscription_txid = Some(final_txid);
      }

      // Prepare for the next transaction in the chain.
      prev_txid = final_txid;
      prev_vout = 0;
      prev_value = Amount::from_sat(carry[tx_idx]);
      prev_script = sender_script.clone(); // carry outputs always use sender's P2PKH
    }

    let txid = inscription_txid.unwrap();
    println!("\nInscription ID: {txid}i0");
    if self.dry_run {
      println!("(dry-run — nothing was broadcast)");
    }

    Ok(None)
  }
}

// ── Script encoding ───────────────────────────────────────────────────────────

/// Encode a data push using the minimal Bitcoin script pushdata opcode.
///
/// - empty  → OP_0  (0x00)
/// - 1–75   → direct pushdata  (opcode = len)
/// - 76–255 → OP_PUSHDATA1  (0x4c + 1-byte len)
/// - 256+   → OP_PUSHDATA2  (0x4d + 2-byte LE len)
fn encode_push(data: &[u8]) -> Vec<u8> {
  let len = data.len();
  let mut out = Vec::with_capacity(len + 3);
  if len == 0 {
    out.push(0x00);
  } else if len <= 75 {
    out.push(len as u8);
    out.extend_from_slice(data);
  } else if len <= 255 {
    out.push(0x4c);
    out.push(len as u8);
    out.extend_from_slice(data);
  } else {
    out.push(0x4d);
    out.push((len & 0xff) as u8);
    out.push((len >> 8) as u8);
    out.extend_from_slice(data);
  }
  out
}

/// Return the serialized byte size of `encode_push(data_of_len)`.
fn encode_push_size(data_len: usize) -> usize {
  if data_len == 0 {
    1
  } else if data_len <= 75 {
    1 + data_len
  } else if data_len <= 255 {
    2 + data_len
  } else {
    3 + data_len
  }
}

/// Encode a remaining_count value using the Doginals multi-part encoding.
///
/// This produces the raw opcode bytes — NOT a pushdata wrapper — mirroring
/// how H3imdall-dev/Dogecoin-Tools encodes the countdown field:
///
/// - 0      → OP_0     (0x00)          — empty push, last chunk
/// - 1–16   → OP_1..OP_16 (0x51..0x60) — small integer in opcode
/// - 17–127 → 1-byte pushdata           — [0x01, n]
/// - 128+   → 2-byte LE pushdata        — [0x02, lo, hi]
///
/// These are exactly the opcodes our scanner in `scan.rs` recognises.
fn encode_count(n: u16) -> Vec<u8> {
  match n {
    0 => vec![0x00],
    1..=16 => vec![0x50 + n as u8],
    17..=127 => vec![0x01, n as u8],
    _ => vec![0x02, (n & 0xff) as u8, (n >> 8) as u8],
  }
}

/// Concatenate pre-encoded script segments and append the sig + pubkey pushes.
fn build_script_sig(segments: &[Vec<u8>], sig: &[u8], pubkey: &[u8]) -> Vec<u8> {
  let total: usize = segments.iter().map(|s| s.len()).sum::<usize>()
    + encode_push_size(sig.len())
    + encode_push_size(pubkey.len());
  let mut out = Vec::with_capacity(total);
  for seg in segments {
    out.extend_from_slice(seg);
  }
  out.extend(encode_push(sig));
  out.extend(encode_push(pubkey));
  out
}

/// Estimate the scriptSig byte size (including the sig + pubkey at the tail).
fn script_sig_size(segments: &[Vec<u8>]) -> usize {
  let segs: usize = segments.iter().map(|s| s.len()).sum();
  // Sig: up to 73 bytes DER + 1 SIGHASH byte = 74; encode_push(74) = 75 bytes.
  // Pubkey: 33 bytes compressed; encode_push(33) = 34 bytes.
  segs + 75 + 34
}

// ── Fee calculation ───────────────────────────────────────────────────────────

/// Calculate the fee in koinu for a single transaction.
///
/// Sizes:
/// - tx overhead  = 10 bytes (version 4 + locktime 4 + 1-byte varint × 2)
/// - per input    = 32 (txid) + 4 (vout) + varint(scriptSig_len) + scriptSig_len + 4 (seq)
/// - per output   = 8 (value) + 1 (varint) + 25 (P2PKH script)
fn calc_fee(scriptSig_bytes: usize, n_outputs: usize, fee_rate: f64) -> u64 {
  let script_varint = if scriptSig_bytes < 0xfd { 1usize } else { 3 };
  let input_size = 32 + 4 + script_varint + scriptSig_bytes + 4;
  let output_size = n_outputs * (8 + 1 + 25);
  let total = 10 + input_size + output_size;
  (total as f64 * fee_rate).ceil() as u64
}

// ── UTXO selection ────────────────────────────────────────────────────────────

/// Pick the largest spendable UTXO (≥1 confirmation) that covers `required` koinu.
fn select_utxo(
  client: &bitcoincore_rpc::Client,
  required: u64,
) -> crate::Result<(Txid, u32, Amount, bitcoin::ScriptBuf)> {
  let utxos = client.list_unspent(Some(1), None, None, None, None)?;

  let utxo = utxos
    .into_iter()
    .filter(|u| u.spendable && u.amount.to_sat() >= required)
    .max_by_key(|u| u.amount)
    .ok_or_else(|| {
      anyhow!(
        "no spendable UTXO covers the required {} koinu ({:.4} DOGE).\n\
         Fund the wallet first: `dog wallet receive` then send DOGE to that address.",
        required,
        required as f64 / 1e8
      )
    })?;

  Ok((utxo.txid, utxo.vout, utxo.amount, utxo.script_pub_key))
}

// ── Signing ───────────────────────────────────────────────────────────────────

/// Sign a template transaction via Core's wallet and return the raw `(sig, pubkey)` bytes.
///
/// The template must have an empty scriptSig.  Core computes the P2PKH sighash
/// over the prevout's scriptPubKey and returns a fully signed transaction.  We
/// extract the `<sig>` and `<pubkey>` pushes from the result — these are then
/// injected into the actual scriptSig (after the inscription data) in the caller.
///
/// The signature is valid for the actual transaction even though the scriptSig
/// will be different: P2PKH sighash covers only the scriptCode (the prevout
/// scriptPubKey), not the scriptSig content.
fn sign_template(
  client: &bitcoincore_rpc::Client,
  tx: &Transaction,
  input_txid: Txid,
  input_vout: u32,
  input_script: &bitcoin::ScriptBuf,
  input_value: Amount,
) -> crate::Result<(Vec<u8>, Vec<u8>)> {
  let hex = hex::encode(serialize(tx));

  let result = client.sign_raw_transaction_with_wallet(
    hex.as_str(),
    Some(&[SignRawTransactionInput {
      txid: input_txid,
      vout: input_vout,
      script_pub_key: input_script.clone(),
      redeem_script: None,
      amount: Some(input_value),
    }]),
    None,
  )?;

  ensure!(
    result.complete,
    "Core could not sign the transaction: {:?}\n\
     Make sure the wallet owns the funding UTXO address.",
    result.errors
  );

  // Deserialise the signed tx and extract sig + pubkey from input[0].script_sig.
  let signed_tx: Transaction = bitcoin::consensus::deserialize(&result.hex)?;
  let script_sig = &signed_tx.input[0].script_sig;

  let mut instructions = script_sig.instructions();

  let sig = match instructions.next() {
    Some(Ok(bitcoin::script::Instruction::PushBytes(pb))) => pb.as_bytes().to_vec(),
    other => bail!(
      "unexpected instruction at sig position in signed scriptSig: {:?}",
      other
    ),
  };

  let pubkey = match instructions.next() {
    Some(Ok(bitcoin::script::Instruction::PushBytes(pb))) => pb.as_bytes().to_vec(),
    other => bail!(
      "unexpected instruction at pubkey position in signed scriptSig: {:?}",
      other
    ),
  };

  Ok((sig, pubkey))
}

// ── Address utilities ─────────────────────────────────────────────────────────

/// Decode a Dogecoin address (P2PKH "D..." or P2SH "A...") into a scriptPubKey.
///
/// Uses the same base58check format as `chain.rs`:
/// - P2PKH version byte 0x1e (30) → "D..." addresses
/// - P2SH  version byte 0x16 (22) → "A..." addresses
fn parse_dogecoin_address(addr: &str) -> crate::Result<bitcoin::ScriptBuf> {
  let decoded = bitcoin::base58::decode_check(addr)
    .map_err(|e| anyhow!("invalid Dogecoin address '{addr}': {e}"))?;

  ensure!(!decoded.is_empty(), "empty payload for address '{addr}'");

  let version = decoded[0];
  let payload = &decoded[1..];

  match version {
    0x1e => {
      // P2PKH — standard "D..." Dogecoin addresses
      let hash = bitcoin::PubkeyHash::from_slice(payload)
        .map_err(|e| anyhow!("invalid P2PKH hash in '{addr}': {e}"))?;
      Ok(bitcoin::ScriptBuf::new_p2pkh(&hash))
    }
    0x16 => {
      // P2SH — "A..." Dogecoin addresses
      let hash = bitcoin::ScriptHash::from_slice(payload)
        .map_err(|e| anyhow!("invalid P2SH hash in '{addr}': {e}"))?;
      Ok(bitcoin::ScriptBuf::new_p2sh(&hash))
    }
    _ => bail!(
      "unsupported address version 0x{version:02x} for '{addr}' \
       (expected P2PKH=0x1e or P2SH=0x16)"
    ),
  }
}

// ── MIME detection ────────────────────────────────────────────────────────────

/// Detect the MIME type of a file using magic bytes first, then extension.
fn detect_mime(path: &Path, data: &[u8]) -> String {
  // Magic byte signatures for common inscription types.
  let by_magic: Option<&str> = if data.starts_with(b"\xff\xd8\xff") {
    Some("image/jpeg")
  } else if data.starts_with(b"\x89PNG\r\n\x1a\n") {
    Some("image/png")
  } else if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
    Some("image/gif")
  } else if data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP" {
    Some("image/webp")
  } else if data.starts_with(b"\x00\x00\x00\x0cjP  ") {
    Some("image/jp2")
  } else if data.starts_with(b"%PDF") {
    Some("application/pdf")
  } else if data.starts_with(b"PK\x03\x04") {
    Some("application/zip")
  } else if data.starts_with(b"\x1f\x8b") {
    Some("application/gzip")
  } else if data.starts_with(b"<svg") || data.starts_with(b"<?xml") {
    Some("image/svg+xml")
  } else {
    None
  };

  if let Some(mime) = by_magic {
    return mime.to_string();
  }

  // Fall back to file extension via mime_guess.
  mime_guess::from_path(path)
    .first_or_octet_stream()
    .essence_str()
    .to_string()
}
