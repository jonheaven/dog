use {
  super::super::*,
  crate::subcommand::dns::DnsInfo,
};

#[derive(Clone, Debug, Parser)]
pub struct ConfigCommand {
  #[arg(help = "Name to show configuration for (e.g., 'satoshi.doge')")]
  pub name: String,

  #[arg(long, help = "Output as JSON")]
  pub json: bool,
}

impl ConfigCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;
    index.update()?;

    if let Some(entry) = index.get_dns_name(&self.name)? {
      if self.json {
        let info = DnsInfo::from(entry);
        println!("{}", serde_json::to_string_pretty(&info)?);
      } else {
        println!("DNS Configuration for '{}':", entry.name);
        println!("  Inscription: {}", entry.owner_inscription_id);
        println!("  Height:      {}", entry.height);
        if let Some(ref addr) = entry.address {
          println!("  Address:     {}", addr);
        }
        if let Some(ref avatar) = entry.avatar {
          println!("  Avatar:      {}", avatar);
        }
        if let Some(ref rev) = entry.reverse {
          println!("  Reverse:     {}", rev);
        }
      }
    } else {
      if self.json {
        println!("{{\"error\": \"name '{}' not found\"}}", self.name);
      } else {
        eprintln!("Name '{}' not found", self.name);
      }
    }

    Ok(None)
  }
}
