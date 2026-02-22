use {
  super::*,
  crate::index::DnsEntry,
  std::collections::HashMap,
};

pub mod config;
pub mod list;
pub mod resolve;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DnsConfig {
  pub p: String,              // Protocol identifier: "dns"
  pub op: String,             // Operation: "config"
  pub name: String,           // Name to configure (e.g., "satoshi.doge")
  pub address: Option<String>, // Dogecoin address to point to
  pub avatar: Option<String>, // Avatar inscription ID or URL
  pub reverse: Option<String>, // Reverse resolution address
}

impl DnsConfig {
  pub fn new_config(name: String, address: Option<String>, avatar: Option<String>) -> Self {
    Self {
      p: "dns".to_string(),
      op: "config".to_string(),
      name,
      address,
      avatar,
      reverse: None,
    }
  }

  pub fn parse_json(json_str: &str) -> Result<Self> {
    let config: DnsConfig = serde_json::from_str(json_str)
      .map_err(|e| anyhow!("Failed to parse DNS config JSON: {}", e))?;

    if config.p != "dns" {
      return Err(anyhow!(
        "Invalid protocol: expected 'dns', got '{}'",
        config.p
      ));
    }

    if config.op != "config" {
      return Err(anyhow!(
        "Invalid operation: expected 'config', got '{}'",
        config.op
      ));
    }

    if config.name.is_empty() {
      return Err(anyhow!("Name cannot be empty"));
    }

    if config.name.contains('.') {
      let parts: Vec<&str> = config.name.split('.').collect();
      if parts.len() == 2 {
        let namespace = parts[1];
        if !is_valid_dns_namespace(namespace) {
          return Err(anyhow!(
            "Invalid namespace '{}'. Supported: .doge, .dogecoin, .shibe, .wow, .very, .such, .much, .woof, .moon, .kabosu, .inu, .doggo",
            namespace
          ));
        }
      }
    }

    if let Some(ref addr) = config.address {
      if addr.len() < 26 || addr.len() > 35 {
        return Err(anyhow!("Invalid Dogecoin address length"));
      }
    }

    Ok(config)
  }
}

/// Returns true if the namespace is a supported Dogecoin name suffix.
pub fn is_valid_dns_namespace(namespace: &str) -> bool {
  matches!(
    namespace,
    "doge"
      | "dogecoin"
      | "shibe"
      | "shib"
      | "wow"
      | "very"
      | "such"
      | "much"
      | "excite"
      | "woof"
      | "bark"
      | "tail"
      | "paws"
      | "paw"
      | "moon"
      | "kabosu"
      | "cheems"
      | "inu"
      | "cook"
      | "doggo"
      | "boop"
      | "zoomies"
      | "smol"
      | "snoot"
      | "pupper"
      | "official"
  )
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DnsStats {
  pub total_names: u64,
  pub names_by_namespace: HashMap<String, u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DnsList {
  pub names: Vec<DnsInfo>,
  pub total: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DnsInfo {
  pub name: String,
  pub owner_inscription_id: String,
  pub owner_inscription_number: i32,
  pub height: u32,
  pub timestamp: u32,
  pub fee: u64,
  pub address: Option<String>,
  pub avatar: Option<String>,
  pub reverse: Option<String>,
}

impl From<DnsEntry> for DnsInfo {
  fn from(entry: DnsEntry) -> Self {
    Self {
      name: entry.name,
      owner_inscription_id: entry.owner_inscription_id.to_string(),
      owner_inscription_number: entry.owner_inscription_number,
      height: entry.height,
      timestamp: entry.timestamp,
      fee: entry.fee,
      address: entry.address,
      avatar: entry.avatar,
      reverse: entry.reverse,
    }
  }
}

#[derive(Clone, Debug, Parser)]
pub struct DnsCommand {
  #[command(subcommand)]
  pub command: DnsSubcommand,
}

#[derive(Clone, Debug, Parser)]
pub enum DnsSubcommand {
  #[command(about = "Resolve a Dogecoin name to an address")]
  Resolve(resolve::ResolveCommand),
  #[command(about = "List registered Dogecoin names")]
  List(list::ListCommand),
  #[command(about = "Show configuration for a Dogecoin name")]
  Config(config::ConfigCommand),
}

impl DnsCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    match self.command {
      DnsSubcommand::Resolve(cmd) => cmd.run(settings),
      DnsSubcommand::List(cmd) => cmd.run(settings),
      DnsSubcommand::Config(cmd) => cmd.run(settings),
    }
  }
}
