use super::*;

pub mod balances;
pub mod decode;
pub mod dns;
pub mod dogemap;
pub mod drc20;
pub mod dune;
pub mod dunes;
pub mod env;
pub mod epochs;
pub mod find;
pub mod index;
pub mod inscribe;
pub mod list;
pub mod parity;
pub mod parse;
pub mod scan;
pub mod server;
mod settings;
pub mod subsidy;
pub mod supply;
pub mod teleburn;
pub mod traits;
pub mod verify;
pub mod wallet;
pub mod wallets;

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  #[command(about = "List all dune balances")]
  Balances,
  #[command(about = "Decode a transaction")]
  Decode(decode::Decode),
  #[command(about = "Dogecoin Name System (DNS) commands")]
  Dns(dns::DnsCommand),
  #[command(about = "Dogemaps block title commands")]
  Dogemap(dogemap::DogemapCommand),
  #[command(about = "DRC-20 token commands")]
  Drc20(drc20::Drc20Command),
  #[command(about = "Dune token commands (list, info, balance)")]
  Dune(dune::DuneCommand),
  #[command(about = "Start a regtest dog and Dogecoin Core instance")]
  Env(env::Env),
  #[command(about = "List the first koinus of each reward epoch")]
  Epochs,
  #[command(about = "Find a koinu's current location")]
  Find(find::Find),
  #[command(subcommand, about = "Index commands")]
  Index(index::IndexSubcommand),
  #[command(about = "Write a Dogecoin inscription to the chain")]
  Inscribe(inscribe::InscribeCommand),
  #[command(about = "List the koinus in an output")]
  List(list::List),
  #[command(about = "Parse a koinu from doginal notation")]
  Parse(parse::Parse),
  #[command(about = "Compare kabosu marketplace/system responses against dog")]
  Parity(parity::Parity),
  #[command(about = "List all dunes")]
  Dunes,
  #[command(about = "Scan a block range for inscriptions (no full index required)")]
  Scan(scan::ScanCommand),
  #[command(about = "Run the explorer server")]
  Server(server::Server),
  #[command(about = "Display settings")]
  Settings,
  #[command(about = "Display information about a block's subsidy")]
  Subsidy(subsidy::Subsidy),
  #[command(about = "Display Dogecoin supply information")]
  Supply,
  #[command(about = "Generate teleburn addresses")]
  Teleburn(teleburn::Teleburn),
  #[command(about = "Display koinu traits")]
  Traits(traits::Traits),
  #[command(about = "Verify BIP322 signature")]
  Verify(verify::Verify),
  #[command(about = "Wallet commands")]
  Wallet(wallet::WalletCommand),
  #[command(about = "List all Dogecoin Core wallets")]
  Wallets,
}

impl Subcommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    match self {
      Self::Balances => balances::run(settings),
      Self::Decode(decode) => decode.run(settings),
      Self::Dns(dns) => dns.run(settings),
      Self::Dogemap(cmd) => cmd.run(settings),
      Self::Drc20(cmd) => cmd.run(settings),
      Self::Dune(cmd) => cmd.run(settings),
      Self::Env(env) => env.run(),
      Self::Epochs => epochs::run(),
      Self::Find(find) => find.run(settings),
      Self::Index(index) => index.run(settings),
      Self::Inscribe(cmd) => cmd.run(settings),
      Self::List(list) => list.run(settings),
      Self::Parse(parse) => parse.run(),
      Self::Parity(parity) => parity.run(settings),
      Self::Dunes => dunes::run(settings),
      Self::Scan(scan) => scan.run(settings),
      Self::Server(server) => {
        let index = Arc::new(Index::open(&settings)?);
        let handle = axum_server::Handle::new();
        LISTENERS.lock().unwrap().push(handle.clone());
        server.run(settings, index, handle, None)
      }
      Self::Settings => settings::run(settings),
      Self::Subsidy(subsidy) => subsidy.run(),
      Self::Supply => supply::run(),
      Self::Teleburn(teleburn) => teleburn.run(),
      Self::Traits(traits) => traits.run(),
      Self::Verify(verify) => verify.run(),
      Self::Wallet(wallet) => wallet.run(settings),
      Self::Wallets => wallets::run(settings),
    }
  }
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum OutputFormat {
  #[default]
  Json,
  Yaml,
  Minify,
}

pub trait Output: Send {
  fn print(&self, format: OutputFormat);
}

impl<T> Output for T
where
  T: Serialize + Send,
{
  fn print(&self, format: OutputFormat) {
    match format {
      OutputFormat::Json => serde_json::to_writer_pretty(io::stdout(), self).ok(),
      OutputFormat::Yaml => serde_yaml::to_writer(io::stdout(), self).ok(),
      OutputFormat::Minify => serde_json::to_writer(io::stdout(), self).ok(),
    };
    println!();
  }
}

pub(crate) type SubcommandResult = Result<Option<Box<dyn Output>>>;
