use {super::*, serde::{Deserialize, Serialize}};

pub mod balance;
pub mod token;
pub mod tokens;

// ---------------------------------------------------------------------------
// Core DRC-20 data structures (shared between indexer and CLI)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Drc20Token {
  /// Original-case 4-character tick (e.g. "DOGI")
  pub tick: String,
  /// Maximum mintable supply, scaled by 10^decimals
  pub max_supply: u128,
  /// Per-mint limit, scaled by 10^decimals
  pub mint_limit: u128,
  /// Decimal places (0–18, default 8)
  pub decimals: u8,
  /// Total amount minted so far, scaled by 10^decimals
  pub minted: u128,
  /// Inscription ID that deployed this token
  pub deploy_inscription: String,
  /// Block height at which the deploy inscription was confirmed
  pub deploy_height: u32,
  /// Block timestamp of the deploy inscription
  pub deploy_timestamp: u32,
  /// Address that created the deploy inscription
  pub deployer: String,
  /// Number of successful mint operations
  pub mint_count: u64,
}

impl Drc20Token {
  /// Returns `minted` as a human-readable decimal string.
  pub fn minted_display(&self) -> String {
    format_amount(self.minted, self.decimals)
  }

  /// Returns `max_supply` as a human-readable decimal string.
  pub fn max_display(&self) -> String {
    format_amount(self.max_supply, self.decimals)
  }

  /// Returns `mint_limit` as a human-readable decimal string.
  pub fn limit_display(&self) -> String {
    format_amount(self.mint_limit, self.decimals)
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Drc20Transfer {
  pub tick: String,
  pub amount: u128,
  pub from_address: String,
}

// ---------------------------------------------------------------------------
// Amount helpers
// ---------------------------------------------------------------------------

/// Format a scaled u128 amount as a decimal string given `decimals` precision.
pub fn format_amount(amount: u128, decimals: u8) -> String {
  if decimals == 0 {
    return amount.to_string();
  }
  let scale = 10u128.pow(decimals as u32);
  let int_part = amount / scale;
  let frac_part = amount % scale;
  if frac_part == 0 {
    int_part.to_string()
  } else {
    let frac_str = format!("{:0>width$}", frac_part, width = decimals as usize);
    let frac_trimmed = frac_str.trim_end_matches('0');
    format!("{}.{}", int_part, frac_trimmed)
  }
}

/// Parse a decimal amount string into a scaled u128.
/// Validates the same rules as the bel-20-indexer:
/// no leading +/-, no leading/trailing dot, no spaces.
pub fn parse_amount(s: &str, decimals: u8) -> Option<u128> {
  let s = s.trim();
  if s.is_empty()
    || s.starts_with('+')
    || s.starts_with('-')
    || s.starts_with('.')
    || s.ends_with('.')
    || s.contains(' ')
  {
    return None;
  }
  let scale = 10u128.pow(decimals as u32);
  if let Some(dot) = s.find('.') {
    let int_part: u128 = s[..dot].parse().ok()?;
    let frac_str = &s[dot + 1..];
    if frac_str.len() > decimals as usize {
      return None; // too many decimal places
    }
    let frac_part: u128 = if frac_str.is_empty() {
      0
    } else {
      frac_str.parse().ok()?
    };
    let frac_scaled =
      frac_part * 10u128.pow((decimals as usize - frac_str.len()) as u32);
    int_part
      .checked_mul(scale)?
      .checked_add(frac_scaled)
  } else {
    s.parse::<u128>().ok()?.checked_mul(scale)
  }
}

/// Read a JSON field that may be either a string or a JSON number.
pub fn json_to_amount_str(v: &serde_json::Value) -> Option<String> {
  if let Some(s) = v.as_str() {
    Some(s.to_string())
  } else if let Some(n) = v.as_u64() {
    Some(n.to_string())
  } else if let Some(n) = v.as_f64() {
    // best-effort for bare floats like 1000.5
    Some(format!("{}", n))
  } else {
    None
  }
}

// ---------------------------------------------------------------------------
// CLI command wiring
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Parser)]
pub struct Drc20Command {
  #[command(subcommand)]
  pub command: Drc20Subcommand,
}

#[derive(Clone, Debug, Parser)]
pub enum Drc20Subcommand {
  #[command(about = "List all deployed DRC-20 tokens")]
  Tokens(tokens::TokensCommand),
  #[command(about = "Show info for a single DRC-20 token")]
  Token(token::TokenCommand),
  #[command(about = "Show DRC-20 balances for a Dogecoin address")]
  Balance(balance::BalanceCommand),
}

impl Drc20Command {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    match self.command {
      Drc20Subcommand::Tokens(cmd) => cmd.run(settings),
      Drc20Subcommand::Token(cmd) => cmd.run(settings),
      Drc20Subcommand::Balance(cmd) => cmd.run(settings),
    }
  }
}
