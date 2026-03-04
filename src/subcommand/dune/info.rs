use super::super::*;

#[derive(Clone, Debug, Parser)]
pub struct InfoCommand {
  #[arg(help = "Dune name to look up (e.g. UNCOMMON•GOODS or UNCOMMONGOODS)")]
  pub name: String,

  #[arg(long, help = "Output as JSON")]
  pub json: bool,
}

impl InfoCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;

    ensure!(
      index.has_dune_index(),
      "`dog dune info` requires index created with `--index-dunes` flag"
    );

    index.update()?;

    let spaced: SpacedDune = self
      .name
      .parse()
      .map_err(|_| anyhow!("invalid dune name: {}", self.name))?;

    match index.dune(spaced.dune)? {
      None => {
        if self.json {
          println!("{{\"error\": \"dune '{}' not found\"}}", self.name);
        } else {
          eprintln!("Dune '{}' not found.", self.name);
        }
      }
      Some((id, entry, parent)) => {
        if self.json {
          println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
              "id": id.to_string(),
              "dune": entry.spaced_dune.to_string(),
              "number": entry.number,
              "supply": entry.supply(),
              "mints": entry.mints,
              "burned": entry.burned,
              "premine": entry.premine,
              "divisibility": entry.divisibility,
              "symbol": entry.symbol.map(|c| c.to_string()),
              "block": id.block,
              "tx": id.tx,
              "etching": entry.etching.to_string(),
              "parent_inscription": parent.map(|p| p.to_string()),
              "turbo": entry.turbo,
              "terms": entry.terms,
            }))?
          );
        } else {
          println!("Dune:        {}", entry.spaced_dune);
          println!("ID:          {}", id);
          println!("Number:      {}", entry.number);
          println!("Supply:      {}", entry.supply());
          println!("Mints:       {}", entry.mints);
          println!("Burned:      {}", entry.burned);
          println!("Premine:     {}", entry.premine);
          println!("Divisibility:{}", entry.divisibility);
          if let Some(sym) = entry.symbol {
            println!("Symbol:      {sym}");
          }
          println!("Block:       {}", id.block);
          println!("Etching tx:  {}", entry.etching);
          if let Some(p) = parent {
            println!("Parent:      {p}");
          }
          println!("Turbo:       {}", entry.turbo);
        }
      }
    }

    Ok(None)
  }
}
