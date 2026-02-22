use super::super::*;

#[derive(Clone, Debug, Parser)]
pub struct TokenCommand {
  #[arg(help = "Tick to look up (e.g. 'dogi')")]
  pub tick: String,

  #[arg(long, help = "Output as JSON")]
  pub json: bool,
}

impl TokenCommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;
    index.update()?;

    if let Some(token) = index.get_drc20_token(&self.tick)? {
      if self.json {
        println!("{}", serde_json::to_string_pretty(&token)?);
      } else {
        println!("Tick:       {}", token.tick);
        println!("Max Supply: {}", token.max_display());
        println!("Minted:     {}", token.minted_display());
        println!("Limit/mint: {}", token.limit_display());
        println!("Decimals:   {}", token.decimals);
        println!("Mints:      {}", token.mint_count);
        println!("Deployer:   {}", token.deployer);
        println!("Deploy ID:  {}", token.deploy_inscription);
        println!("Height:     {}", token.deploy_height);
      }
    } else {
      if self.json {
        println!("{{\"error\": \"token '{}' not found\"}}", self.tick);
      } else {
        eprintln!("Token '{}' not found", self.tick);
      }
    }

    Ok(None)
  }
}
