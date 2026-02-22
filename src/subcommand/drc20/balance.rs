use {super::super::*, crate::subcommand::drc20::format_amount};

#[derive(Clone, Debug, Parser)]
pub struct BalanceCommand {
  #[arg(help = "Dogecoin address to check")]
  pub address: String,

  #[arg(long, help = "Filter to a single tick (e.g. 'dogi')")]
  pub tick: Option<String>,

  #[arg(long, help = "Output as JSON")]
  pub json: bool,
}

impl BalanceCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;
    index.update()?;

    if let Some(ref tick) = self.tick {
      // Single token balance
      let (available, transferable) = index.get_drc20_balance(&self.address, tick)?;
      let total = available + transferable;

      // Get decimals for display
      let decimals = index
        .get_drc20_token(tick)?
        .map(|t| t.decimals)
        .unwrap_or(8);

      if self.json {
        let out = serde_json::json!({
          "address": self.address,
          "tick": tick.to_lowercase(),
          "available": format_amount(available, decimals),
          "transferable": format_amount(transferable, decimals),
          "total": format_amount(total, decimals),
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
      } else {
        println!("Address:     {}", self.address);
        println!("Tick:        {}", tick.to_lowercase());
        println!("Available:   {}", format_amount(available, decimals));
        println!("Transferable:{}", format_amount(transferable, decimals));
        println!("Total:       {}", format_amount(total, decimals));
      }
    } else {
      // All token balances for address
      let balances = index.get_drc20_balances(&self.address)?;

      if self.json {
        let rows: Vec<_> = balances
          .iter()
          .map(|(tick, avail, trf)| {
            let decimals = 8u8; // default; ideally look up per token
            serde_json::json!({
              "tick": tick,
              "available": format_amount(*avail, decimals),
              "transferable": format_amount(*trf, decimals),
              "total": format_amount(avail + trf, decimals),
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
        println!("DRC-20 balances for {}", self.address);
        println!("{:<6} {:>18} {:>18} {:>18}", "Tick", "Available", "Transferable", "Total");
        println!("{}", "-".repeat(65));
        for (tick, avail, trf) in &balances {
          let decimals = 8u8;
          println!(
            "{:<6} {:>18} {:>18} {:>18}",
            tick,
            format_amount(*avail, decimals),
            format_amount(*trf, decimals),
            format_amount(avail + trf, decimals),
          );
        }
        if balances.is_empty() {
          println!("No DRC-20 tokens found.");
        }
      }
    }

    Ok(None)
  }
}
