use {super::super::*, crate::subcommand::drc20::format_amount};

#[derive(Clone, Debug, Parser)]
pub struct TokensCommand {
  #[arg(long, help = "Output as JSON")]
  pub json: bool,
}

impl TokensCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;
    index.update()?;

    let mut tokens = index.get_drc20_tokens()?;
    tokens.sort_by(|a, b| a.tick.to_lowercase().cmp(&b.tick.to_lowercase()));

    if self.json {
      println!("{}", serde_json::to_string_pretty(&tokens)?);
    } else {
      println!(
        "{:<6} {:>18} {:>18} {:>6} {:>10} {}",
        "Tick", "Supply", "Max", "Dec", "Mints", "Deploy Inscription"
      );
      println!("{}", "-".repeat(90));
      for t in &tokens {
        println!(
          "{:<6} {:>18} {:>18} {:>6} {:>10} {}",
          t.tick,
          format_amount(t.minted, t.decimals),
          format_amount(t.max_supply, t.decimals),
          t.decimals,
          t.mint_count,
          t.deploy_inscription,
        );
      }
      println!("\nTotal tokens: {}", tokens.len());
    }

    Ok(None)
  }
}
