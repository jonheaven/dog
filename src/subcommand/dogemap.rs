use {super::*, crate::index::DogemapEntry};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DogemapInfo {
  pub block_number: u32,
  pub owner_inscription_id: String,
  pub claim_height: u32,
  pub claim_timestamp: u32,
}

impl From<DogemapEntry> for DogemapInfo {
  fn from(e: DogemapEntry) -> Self {
    Self {
      block_number: e.block_number,
      owner_inscription_id: e.owner_inscription_id.to_string(),
      claim_height: e.claim_height,
      claim_timestamp: e.claim_timestamp,
    }
  }
}

#[derive(Clone, Debug, Parser)]
pub struct DogemapCommand {
  #[command(subcommand)]
  pub command: DogemapSubcommand,
}

#[derive(Clone, Debug, Parser)]
pub enum DogemapSubcommand {
  #[command(about = "Show claim status for a block number")]
  Status(StatusCommand),
  #[command(about = "List all claimed block numbers")]
  List(ListCommand),
}

impl DogemapCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    match self.command {
      DogemapSubcommand::Status(cmd) => cmd.run(settings),
      DogemapSubcommand::List(cmd) => cmd.run(settings),
    }
  }
}

// ---------------------------------------------------------------------------
// dogemap status <block_number>
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Parser)]
pub struct StatusCommand {
  #[arg(help = "Block number to query (e.g. 5056597)")]
  pub block_number: u32,
}

impl StatusCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;
    index.update()?;

    if let Some(entry) = index.get_dogemap_claim(self.block_number)? {
      let info = DogemapInfo::from(entry);
      println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
      println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
          "block_number": self.block_number,
          "claimed": false
        }))?
      );
    }

    Ok(None)
  }
}

// ---------------------------------------------------------------------------
// dogemap list [--limit N] [--offset N]
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Parser)]
pub struct ListCommand {
  #[arg(long, default_value = "100", help = "Maximum number of results")]
  pub limit: usize,

  #[arg(long, default_value = "0", help = "Skip this many results")]
  pub offset: usize,
}

impl ListCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;
    index.update()?;

    let entries = index.list_dogemaps(self.limit, self.offset)?;
    let total = index.count_dogemaps()?;
    let infos: Vec<DogemapInfo> = entries.into_iter().map(DogemapInfo::from).collect();

    println!(
      "{}",
      serde_json::to_string_pretty(&serde_json::json!({
        "total": total,
        "offset": self.offset,
        "limit": self.limit,
        "claims": infos,
      }))?
    );

    Ok(None)
  }
}
