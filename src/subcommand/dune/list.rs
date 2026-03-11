use super::super::*;

#[derive(Clone, Debug, Parser)]
pub struct ListCommand {
  #[arg(long, help = "Output as JSON")]
  pub json: bool,
}

impl ListCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;

    ensure!(
      index.has_dune_index(),
      "`dog dune list` requires index created with `--index-dunes` flag"
    );

    index.update()?;

    let entries = index.dunes()?;

    if self.json {
      let rows: Vec<serde_json::Value> = entries
        .into_iter()
        .map(|(id, entry)| {
          serde_json::json!({
            "id": id.to_string(),
            "dune": entry.spaced_dune.to_string(),
            "supply": entry.supply(),
            "mints": entry.mints,
            "burned": entry.burned,
            "divisibility": entry.divisibility,
            "symbol": entry.symbol.map(|c| c.to_string()),
            "block": id.block,
            "etching": entry.etching.to_string(),
            "turbo": entry.turbo,
          })
        })
        .collect();
      println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
      println!(
        "{:<30} {:>14} {:>10} {:>10}",
        "Dune", "Supply", "Mints", "Block"
      );
      println!("{}", "-".repeat(67));
      for (id, entry) in &entries {
        println!(
          "{:<30} {:>14} {:>10} {:>10}",
          entry.spaced_dune,
          entry.supply(),
          entry.mints,
          id.block,
        );
      }
      println!("\n{} dune(s) total.", entries.len());
    }

    Ok(None)
  }
}
