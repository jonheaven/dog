use super::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PendingOutput {
  pub commit: Txid,
  pub dune: SpacedDune,
}
#[derive(Debug, Parser)]
pub(crate) struct Pending {}

impl Pending {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let etchings = wallet
      .pending_etchings()?
      .into_iter()
      .map(|(_, entry)| {
        let spaced_dune = entry.output.dune.unwrap().dune;

        PendingOutput {
          dune: spaced_dune,
          commit: entry.commit.compute_txid(),
        }
      })
      .collect::<Vec<PendingOutput>>();

    Ok(Some(Box::new(etchings) as Box<dyn Output>))
  }
}
