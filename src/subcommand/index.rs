use super::*;

mod export;
pub mod info;
mod refresh_blk_index;
mod update;

#[derive(Debug, Parser)]
pub(crate) enum IndexSubcommand {
  #[command(about = "Write inscription numbers and ids to a tab-separated file")]
  Export(export::Export),
  #[command(about = "Print index statistics")]
  Info(info::Info),
  #[command(about = "Refresh the shadow copy of Core's block index (safe while Core runs)")]
  RefreshBlkIndex,
  #[command(about = "Update the index", alias = "run")]
  Update,
}

impl IndexSubcommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    match self {
      Self::Export(export) => export.run(settings),
      Self::Info(info) => info.run(settings),
      Self::RefreshBlkIndex => refresh_blk_index::run(settings),
      Self::Update => update::run(settings),
    }
  }
}
