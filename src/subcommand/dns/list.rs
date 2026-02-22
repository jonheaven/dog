use {
  super::super::*,
  crate::subcommand::dns::{DnsInfo, DnsList},
};

#[derive(Clone, Debug, Parser)]
pub struct ListCommand {
  #[arg(long, help = "Filter by namespace (e.g., 'doge')")]
  pub namespace: Option<String>,

  #[arg(long, help = "Maximum number of results to display")]
  pub limit: Option<usize>,

  #[arg(long, help = "Output as JSON")]
  pub json: bool,
}

impl ListCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;
    index.update()?;

    let mut names: Vec<DnsInfo> = Vec::new();

    if let Some(ref namespace) = self.namespace {
      if let Some(ns_names) = index.get_dns_names_by_namespace(namespace)? {
        for name in ns_names {
          if let Some(entry) = index.get_dns_name(&name)? {
            names.push(DnsInfo::from(entry));
          }
        }
      }
    }

    let total = names.len() as u64;

    if let Some(limit) = self.limit {
      names.truncate(limit);
    }

    let list = DnsList { names, total };

    if self.json {
      println!("{}", serde_json::to_string_pretty(&list)?);
    } else {
      println!("DNS Names (Total: {})", list.total);
      println!(
        "{:<30} {:<35} {:<8} {}",
        "Name", "Address", "Inscription#", "Height"
      );
      println!("{}", "-".repeat(85));
      for info in &list.names {
        let address = info.address.as_deref().unwrap_or("(not configured)");
        println!(
          "{:<30} {:<35} {:<8} {}",
          info.name, address, info.owner_inscription_number, info.height
        );
      }
    }

    Ok(None)
  }
}
