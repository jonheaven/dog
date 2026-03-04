use super::*;

pub mod balance;
pub mod info;
pub mod list;

#[derive(Clone, Debug, Parser)]
pub struct DuneCommand {
  #[command(subcommand)]
  pub command: DuneSubcommand,
}

#[derive(Clone, Debug, Parser)]
pub enum DuneSubcommand {
  #[command(about = "List all etched dunes")]
  List(list::ListCommand),
  #[command(about = "Show info for a specific dune")]
  Info(info::InfoCommand),
  #[command(about = "Show dune balances for a Dogecoin address")]
  Balance(balance::BalanceCommand),
}

impl DuneCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    match self.command {
      DuneSubcommand::List(cmd) => cmd.run(settings),
      DuneSubcommand::Info(cmd) => cmd.run(settings),
      DuneSubcommand::Balance(cmd) => cmd.run(settings),
    }
  }
}
