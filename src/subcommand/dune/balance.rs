use {super::super::*, crate::subcommand::inscribe::parse_dogecoin_address};

#[derive(Clone, Debug, Parser)]
pub struct BalanceCommand {
  #[arg(help = "Dogecoin address to check (e.g. D8jt...)")]
  pub address: String,

  #[arg(long, help = "Output as JSON")]
  pub json: bool,
}

impl BalanceCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;

    ensure!(
      index.has_dune_index(),
      "`dog dune balance` requires index created with `--index-dunes` flag"
    );

    ensure!(
      index.has_address_index(),
      "`dog dune balance` requires index created with `--index-addresses` flag"
    );

    index.update()?;

    let script = parse_dogecoin_address(&self.address)?;
    let balances = index.get_dune_balances_for_script(script.as_bytes())?;

    if self.json {
      let rows: Vec<serde_json::Value> = balances
        .iter()
        .map(|(dune, amount, div, sym)| {
          serde_json::json!({
            "dune": dune.to_string(),
            "amount": amount,
            "display": format_pile(*amount, *div, *sym),
            "divisibility": div,
            "symbol": sym.map(|c| c.to_string()),
          })
        })
        .collect();
      println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
          "address": self.address,
          "balances": rows,
        }))?
      );
    } else {
      println!("Dune balances for {}", self.address);
      println!("{:<30} {:>20}", "Dune", "Balance");
      println!("{}", "-".repeat(52));
      for (dune, amount, div, sym) in &balances {
        println!("{:<30} {:>20}", dune, format_pile(*amount, *div, *sym));
      }
      if balances.is_empty() {
        println!("No dune balances found.");
      }
    }

    Ok(None)
  }
}

fn format_pile(amount: u128, divisibility: u8, symbol: Option<char>) -> String {
  let sym = symbol.unwrap_or('¤');
  if divisibility == 0 {
    return format!("{amount}\u{A0}{sym}");
  }
  let scale = 10u128.pow(divisibility as u32);
  let whole = amount / scale;
  let frac = amount % scale;
  if frac == 0 {
    format!("{whole}\u{A0}{sym}")
  } else {
    format!("{whole}.{frac:0>width$}\u{A0}{sym}", width = divisibility as usize)
  }
}
