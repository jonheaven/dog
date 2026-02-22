use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Mint {
  #[clap(long, help = "Use <FEE_RATE> koinu/vbyte for mint transaction.")]
  fee_rate: FeeRate,
  #[clap(long, help = "Mint <RUNE>. May contain `.` or `•`as spacers.")]
  dune: SpacedDune,
  #[clap(
    long,
    help = "Include <AMOUNT> postage with mint output. [default: 10000sat]"
  )]
  postage: Option<Amount>,
  #[clap(long, help = "Send minted dunes to <DESTINATION>.")]
  destination: Option<Address<NetworkUnchecked>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
  pub dune: SpacedDune,
  pub pile: Pile,
  pub mint: Txid,
}

impl Mint {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    ensure!(
      wallet.has_rune_index(),
      "`ord wallet mint` requires index created with `--index-dunes` flag",
    );

    let dune = self.dune.dune;

    let bitcoin_client = wallet.bitcoin_client();

    let block_height = bitcoin_client.get_block_count()?;

    let Some((id, rune_entry, _)) = wallet.get_rune(dune)? else {
      bail!("dune {dune} has not been etched");
    };

    let postage = self.postage.unwrap_or(TARGET_POSTAGE);

    let amount = rune_entry
      .mintable(block_height + 1)
      .map_err(|err| anyhow!("dune {dune} {err}"))?;

    let chain = wallet.chain();

    let destination = match self.destination {
      Some(destination) => destination.require_network(chain.network())?,
      None => wallet.get_change_address()?,
    };

    ensure!(
      destination.script_pubkey().minimal_non_dust() <= postage,
      "postage below dust limit of {}sat",
      destination.script_pubkey().minimal_non_dust().to_sat()
    );

    let dunestone = Dunestone {
      mint: Some(id),
      ..default()
    };

    let script_pubkey = dunestone.encipher();

    ensure!(
      script_pubkey.len() <= MAX_STANDARD_OP_RETURN_SIZE,
      "dunestone greater than maximum OP_RETURN size: {} > {}",
      script_pubkey.len(),
      MAX_STANDARD_OP_RETURN_SIZE,
    );

    let unfunded_transaction = Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: Vec::new(),
      output: vec![
        TxOut {
          script_pubkey,
          value: Amount::from_sat(0),
        },
        TxOut {
          script_pubkey: destination.script_pubkey(),
          value: postage,
        },
      ],
    };

    wallet.lock_non_cardinal_outputs()?;

    let unsigned_transaction =
      fund_raw_transaction(bitcoin_client, self.fee_rate, &unfunded_transaction, None)?;

    let signed_transaction = bitcoin_client
      .sign_raw_transaction_with_wallet(&unsigned_transaction, None, None)?
      .hex;

    let signed_transaction = consensus::encode::deserialize(&signed_transaction)?;

    assert_eq!(
      Dunestone::decipher(&signed_transaction),
      Some(Artifact::Dunestone(dunestone)),
    );

    let transaction = bitcoin_client.send_raw_transaction(&signed_transaction)?;

    Ok(Some(Box::new(Output {
      dune: self.dune,
      pile: Pile {
        amount,
        divisibility: rune_entry.divisibility,
        symbol: rune_entry.symbol,
      },
      mint: transaction,
    })))
  }
}
