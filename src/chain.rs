use {super::*, clap::ValueEnum};

#[derive(Default, ValueEnum, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Chain {
  #[default]
  #[value(alias("doge"))]
  Dogecoin,
  #[value(alias("doge-testnet"))]
  DogecoinTestnet,
  DogecoinRegtest,
}

impl Chain {
  pub(crate) fn network(self) -> Network {
    self.into()
  }

  pub(crate) fn is_dogecoin(self) -> bool {
    true // all chains are Dogecoin
  }

  pub(crate) fn default_rpc_port(self) -> u16 {
    match self {
      Self::Dogecoin => 22555,
      Self::DogecoinTestnet => 44555,
      Self::DogecoinRegtest => 18444,
    }
  }

  pub(crate) fn inscription_content_size_limit(self) -> Option<usize> {
    match self {
      Self::Dogecoin | Self::DogecoinRegtest => None,
      Self::DogecoinTestnet => Some(1024),
    }
  }

  pub(crate) fn first_inscription_height(self) -> u32 {
    match self {
      Self::Dogecoin => 4_600_000,
      Self::DogecoinTestnet => 4_250_000,
      Self::DogecoinRegtest => 0,
    }
  }

  pub(crate) fn first_rune_height(self) -> u32 {
    // "Dunes" (Dogecoin Dunes protocol) — use u32::MAX until activated
    u32::MAX
  }

  pub(crate) fn jubilee_height(self) -> u32 {
    // Dogecoin has no jubilee; treat the same as first inscription height
    match self {
      Self::DogecoinRegtest => 0,
      _ => self.first_inscription_height(),
    }
  }

  pub(crate) fn genesis_block(self) -> Block {
    let genesis_hex: &str = match self {
      Self::Dogecoin => {
        "010000000000000000000000000000000000000000000000000000000000000000000000696ad20e2dd4365c7459b4a4a5af743d5e92c6da3229e6532cd605f6533f2a5b24a6a152f0ff0f1e678601000101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff1004ffff001d0104084e696e746f6e646fffffffff010058850c020000004341040184710fa689ad5023690c80f3a49c8f13f8d45b8c857fbcbc8bc4a8e4d3eb4b10f4d4604fa08dce601aaf0f470216fe1b51850b4acf21b179c45070ac7b03a9ac00000000"
      }
      Self::DogecoinRegtest => {
        "010000000000000000000000000000000000000000000000000000000000000000000000696ad20e2dd4365c7459b4a4a5af743d5e92c6da3229e6532cd605f6533f2a5bdae5494dffff7f20020000000101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff1004ffff001d0104084e696e746f6e646fffffffff010058850c020000004341040184710fa689ad5023690c80f3a49c8f13f8d45b8c857fbcbc8bc4a8e4d3eb4b10f4d4604fa08dce601aaf0f470216fe1b51850b4acf21b179c45070ac7b03a9ac00000000"
      }
      Self::DogecoinTestnet => {
        "010000000000000000000000000000000000000000000000000000000000000000000000696ad20e2dd4365c7459b4a4a5af743d5e92c6da3229e6532cd605f6533f2a5bb9a7f052f0ff0f1ef7390f000101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff1004ffff001d0104084e696e746f6e646fffffffff010058850c020000004341040184710fa689ad5023690c80f3a49c8f13f8d45b8c857fbcbc8bc4a8e4d3eb4b10f4d4604fa08dce601aaf0f470216fe1b51850b4acf21b179c45070ac7b03a9ac00000000"
      }
    };
    let genesis_buf: Vec<u8> = hex::decode(genesis_hex).expect("valid genesis hex");
    bitcoin::consensus::deserialize(&genesis_buf).expect("valid genesis block")
  }

  pub(crate) fn genesis_coinbase_outpoint(self) -> OutPoint {
    OutPoint {
      txid: self.genesis_block().coinbase().unwrap().compute_txid(),
      vout: 0,
    }
  }

  pub(crate) fn address_from_script(self, script: &Script) -> Result<Address, SnafuError> {
    // Dogecoin uses Bitcoin's P2PKH/P2SH script format but with different
    // base58check version bytes.  We decode via Network::Bitcoin so that
    // script parsing works, then callers that need the address string should
    // use address_string_from_script() for the correct "D..." / "A..." form.
    Address::from_script(script, Network::Bitcoin).snafu_context(error::AddressConversion)
  }

  /// Returns the correct Dogecoin address string from a script, using
  /// Dogecoin's base58check version bytes (P2PKH=0x1e → "D...", P2SH=0x16 → "A...").
  pub(crate) fn address_string_from_script(self, script: &Script) -> Option<String> {
    dogecoin_address_string(script)
  }

  pub(crate) fn join_with_data_dir(self, data_dir: impl AsRef<Path>) -> PathBuf {
    match self {
      Self::Dogecoin => data_dir.as_ref().to_owned(),
      Self::DogecoinTestnet => data_dir.as_ref().join("testnet3"),
      Self::DogecoinRegtest => data_dir.as_ref().join("regtest"),
    }
  }
}

/// Encode a Dogecoin address from a script using base58check with
/// Dogecoin-specific version bytes: P2PKH=0x1e (30), P2SH=0x16 (22).
fn dogecoin_address_string(script: &Script) -> Option<String> {
  let bytes = script.as_bytes();
  if script.is_p2pkh() && bytes.len() == 25 {
    // OP_DUP OP_HASH160 <20-byte-hash> OP_EQUALVERIFY OP_CHECKSIG
    Some(dogecoin_base58check(0x1e, &bytes[3..23]))
  } else if script.is_p2sh() && bytes.len() == 23 {
    // OP_HASH160 <20-byte-hash> OP_EQUAL
    Some(dogecoin_base58check(0x16, &bytes[2..22]))
  } else {
    None
  }
}

fn dogecoin_base58check(version: u8, payload: &[u8]) -> String {
  let mut data = Vec::with_capacity(1 + payload.len());
  data.push(version);
  data.extend_from_slice(payload);
  bitcoin::base58::encode_check(&data)
}

impl From<Chain> for Network {
  fn from(chain: Chain) -> Network {
    match chain {
      // Dogecoin uses the same wire format as Bitcoin; Network::Bitcoin
      // is used so that block/tx deserialization works correctly.
      Chain::Dogecoin => Network::Bitcoin,
      Chain::DogecoinTestnet => Network::Testnet,
      Chain::DogecoinRegtest => Network::Regtest,
    }
  }
}

impl Display for Chain {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Dogecoin => "dogecoin",
        Self::DogecoinTestnet => "dogecoin-testnet",
        Self::DogecoinRegtest => "dogecoin-regtest",
      }
    )
  }
}

impl FromStr for Chain {
  type Err = SnafuError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "dogecoin" | "doge" | "mainnet" => Ok(Self::Dogecoin),
      "dogecoin-testnet" | "doge-testnet" | "testnet" => Ok(Self::DogecoinTestnet),
      "dogecoin-regtest" | "doge-regtest" | "regtest" => Ok(Self::DogecoinRegtest),
      _ => Err(SnafuError::InvalidChain {
        chain: s.to_string(),
      }),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_str() {
    assert_eq!("dogecoin".parse::<Chain>().unwrap(), Chain::Dogecoin);
    assert_eq!("doge".parse::<Chain>().unwrap(), Chain::Dogecoin);
    assert_eq!(
      "dogecoin-testnet".parse::<Chain>().unwrap(),
      Chain::DogecoinTestnet
    );
    assert_eq!(
      "dogecoin-regtest".parse::<Chain>().unwrap(),
      Chain::DogecoinRegtest
    );
    assert_eq!(
      "foo".parse::<Chain>().unwrap_err().to_string(),
      "Invalid chain `foo`"
    );
  }

  #[test]
  fn all_chains_are_dogecoin() {
    assert!(Chain::Dogecoin.is_dogecoin());
    assert!(Chain::DogecoinTestnet.is_dogecoin());
    assert!(Chain::DogecoinRegtest.is_dogecoin());
  }

  #[test]
  fn rpc_ports() {
    assert_eq!(Chain::Dogecoin.default_rpc_port(), 22555);
    assert_eq!(Chain::DogecoinTestnet.default_rpc_port(), 44555);
    assert_eq!(Chain::DogecoinRegtest.default_rpc_port(), 18444);
  }

  #[test]
  fn genesis_blocks_parse() {
    let _ = Chain::Dogecoin.genesis_block();
    let _ = Chain::DogecoinRegtest.genesis_block();
    let _ = Chain::DogecoinTestnet.genesis_block();
  }
}
