use {super::*, crate::wallet::Maturity};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ResumeOutput {
  pub etchings: Vec<batch::Output>,
}
#[derive(Debug, Parser)]
pub(crate) struct Resume {
  #[arg(long, help = "Don't broadcast transactions.")]
  pub(crate) dry_run: bool,
  #[arg(long, help = "Pending <RUNE> etching to resume.")]
  pub(crate) dune: Option<SpacedDune>,
}

impl Resume {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let mut etchings = Vec::new();
    loop {
      if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
        break;
      }

      let spaced_dune = self.dune;

      let pending_etchings = if let Some(spaced_dune) = spaced_dune {
        let pending_etching = wallet.load_etching(spaced_dune.dune)?;

        ensure!(
          pending_etching.is_some(),
          "dune {spaced_dune} does not correspond to any pending etching."
        );

        vec![(spaced_dune.dune, pending_etching.unwrap())]
      } else {
        wallet.pending_etchings()?
      };

      for (dune, entry) in pending_etchings {
        if self.dry_run {
          etchings.push(batch::Output {
            reveal_broadcast: false,
            ..entry.output.clone()
          });
          continue;
        };

        match wallet.check_maturity(dune, &entry.commit)? {
          Maturity::Mature => etchings.push(wallet.send_etching(dune, &entry)?),
          Maturity::CommitSpent(txid) => {
            eprintln!("Commitment for dune etching {dune} spent in {txid}");
            wallet.clear_etching(dune)?;
          }
          Maturity::CommitNotFound => {}
          Maturity::BelowMinimumHeight(_) => {}
          Maturity::ConfirmationsPending(_) => {}
        }
      }

      if wallet.pending_etchings()?.is_empty() {
        break;
      }

      if self.dry_run {
        break;
      }

      if !wallet.integration_test() {
        thread::sleep(Duration::from_secs(5));
      }
    }

    Ok(Some(Box::new(ResumeOutput { etchings }) as Box<dyn Output>))
  }
}
