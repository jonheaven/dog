use {super::*, clap::ValueEnum};

#[derive(Default, ValueEnum, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Chain {
  #[default]
  #[value(alias("main"))]
  Mainnet,
  Regtest,
  Signet,
  #[value(alias("test"))]
  Testnet,
  Testnet4,
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

  pub(crate) fn bech32_hrp(self) -> KnownHrp {
    match self {
      Self::Mainnet | Self::Dogecoin => KnownHrp::Mainnet,
      Self::Regtest | Self::DogecoinRegtest => KnownHrp::Regtest,
      Self::Signet | Self::Testnet | Self::Testnet4 | Self::DogecoinTestnet => KnownHrp::Testnets,
    }
  }

  pub(crate) fn is_dogecoin(self) -> bool {
    matches!(
      self,
      Self::Dogecoin | Self::DogecoinTestnet | Self::DogecoinRegtest
    )
  }

  pub(crate) fn default_rpc_port(self) -> u16 {
    match self {
      Self::Mainnet => 8332,
      Self::Regtest => 18443,
      Self::Signet => 38332,
      Self::Testnet => 18332,
      Self::Testnet4 => 48332,
      Self::Dogecoin => 22555,
      Self::DogecoinTestnet => 44555,
      Self::DogecoinRegtest => 18444,
    }
  }

  pub(crate) fn inscription_content_size_limit(self) -> Option<usize> {
    match self {
      Self::Mainnet | Self::Regtest | Self::Dogecoin | Self::DogecoinRegtest => None,
      Self::Testnet | Self::Testnet4 | Self::Signet | Self::DogecoinTestnet => Some(1024),
    }
  }

  pub(crate) fn first_inscription_height(self) -> u32 {
    match self {
      Self::Mainnet => 767430,
      Self::Regtest => 0,
      Self::Signet => 112402,
      Self::Testnet => 2413343,
      Self::Testnet4 => 0,
      Self::Dogecoin => 4_600_000,
      Self::DogecoinTestnet => 4_250_000,
      Self::DogecoinRegtest => 0,
    }
  }

  pub(crate) fn first_rune_height(self) -> u32 {
    if self.is_dogecoin() {
      // "Dunes" (Dogecoin runes equivalent) — use u32::MAX until activated
      u32::MAX
    } else {
      Rune::first_rune_height(self.into())
    }
  }

  pub(crate) fn jubilee_height(self) -> u32 {
    match self {
      Self::Mainnet => 824544,
      Self::Regtest => 110,
      Self::Signet => 175392,
      Self::Testnet => 2544192,
      Self::Testnet4 => 0,
      // Dogecoin has no jubilee; treat it the same as first inscription height
      Self::Dogecoin => self.first_inscription_height(),
      Self::DogecoinTestnet => self.first_inscription_height(),
      Self::DogecoinRegtest => 0,
    }
  }

  pub(crate) fn genesis_block(self) -> Block {
    if self.is_dogecoin() {
      let genesis_hex: &str = match self {
        Self::Dogecoin => {
          "010000000000000000000000000000000000000000000000000000000000000000000000696ad20e2dd4365c7459b4a4a5af743d5e92c6da3229e6532cd605f6533f2a5b24a6a152f0ff0f1e678601000101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff1004ffff001d0104084e696e746f6e646fffffffff010058850c020000004341040184710fa689ad5023690c80f3a49c8f13f8d45b8c857fbcbc8bc4a8e4d3eb4b10f4d4604fa08dce601aaf0f470216fe1b51850b4acf21b179c45070ac7b03a9ac00000000"
        }
        Self::DogecoinRegtest => {
          "010000000000000000000000000000000000000000000000000000000000000000000000696ad20e2dd4365c7459b4a4a5af743d5e92c6da3229e6532cd605f6533f2a5bdae5494dffff7f20020000000101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff1004ffff001d0104084e696e746f6e646fffffffff010058850c020000004341040184710fa689ad5023690c80f3a49c8f13f8d45b8c857fbcbc8bc4a8e4d3eb4b10f4d4604fa08dce601aaf0f470216fe1b51850b4acf21b179c45070ac7b03a9ac00000000"
        }
        // testnet uses same genesis structure
        _ => {
          "010000000000000000000000000000000000000000000000000000000000000000000000696ad20e2dd4365c7459b4a4a5af743d5e92c6da3229e6532cd605f6533f2a5bb9a7f052f0ff0f1ef7390f000101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff1004ffff001d0104084e696e746f6e646fffffffff010058850c020000004341040184710fa689ad5023690c80f3a49c8f13f8d45b8c857fbcbc8bc4a8e4d3eb4b10f4d4604fa08dce601aaf0f470216fe1b51850b4acf21b179c45070ac7b03a9ac00000000"
        }
      };
      let genesis_buf: Vec<u8> = hex::decode(genesis_hex).expect("valid genesis hex");
      bitcoin::consensus::deserialize(&genesis_buf).expect("valid genesis block")
    } else {
      bitcoin::blockdata::constants::genesis_block(self.network())
    }
  }

  pub(crate) fn genesis_coinbase_outpoint(self) -> OutPoint {
    OutPoint {
      txid: self.genesis_block().coinbase().unwrap().compute_txid(),
      vout: 0,
    }
  }

  pub(crate) fn address_from_script(self, script: &Script) -> Result<Address, SnafuError> {
    if self.is_dogecoin() {
      // Dogecoin uses Bitcoin's P2PKH/P2SH script format but with different
      // base58check version bytes: P2PKH=0x1e ("D..."), P2SH=0x16 ("A...").
      // We construct and validate the script type, then encode manually.
      Address::from_script(script, Network::Bitcoin)
        .snafu_context(error::AddressConversion)
    } else {
      Address::from_script(script, self.network()).snafu_context(error::AddressConversion)
    }
  }

  /// Returns the correct Dogecoin address string from a script, using
  /// Dogecoin's base58check version bytes. For non-Dogecoin chains,
  /// returns the standard bitcoin Address display string.
  pub(crate) fn address_string_from_script(self, script: &Script) -> Option<String> {
    if self.is_dogecoin() {
      dogecoin_address_string(script)
    } else {
      Address::from_script(script, self.network())
        .ok()
        .map(|a| a.to_string())
    }
  }

  pub(crate) fn join_with_data_dir(self, data_dir: impl AsRef<Path>) -> PathBuf {
    match self {
      Self::Mainnet => data_dir.as_ref().to_owned(),
      Self::Regtest => data_dir.as_ref().join("regtest"),
      Self::Signet => data_dir.as_ref().join("signet"),
      Self::Testnet => data_dir.as_ref().join("testnet3"),
      Self::Testnet4 => data_dir.as_ref().join("testnet4"),
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
  // encode_check prepends nothing — we prefix the version byte ourselves then
  // let bitcoin::base58::encode_check append the 4-byte SHA256d checksum.
  let mut data = Vec::with_capacity(1 + payload.len());
  data.push(version);
  data.extend_from_slice(payload);
  bitcoin::base58::encode_check(&data)
}

impl From<Chain> for Network {
  fn from(chain: Chain) -> Network {
    match chain {
      Chain::Mainnet => Network::Bitcoin,
      Chain::Regtest => Network::Regtest,
      Chain::Signet => Network::Signet,
      Chain::Testnet => Network::Testnet,
      Chain::Testnet4 => Network::Testnet4,
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
        Self::Mainnet => "mainnet",
        Self::Regtest => "regtest",
        Self::Signet => "signet",
        Self::Testnet => "testnet",
        Self::Testnet4 => "testnet4",
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
      "mainnet" => Ok(Self::Mainnet),
      "regtest" => Ok(Self::Regtest),
      "signet" => Ok(Self::Signet),
      "testnet" => Ok(Self::Testnet),
      "testnet4" => Ok(Self::Testnet4),
      "dogecoin" | "doge" => Ok(Self::Dogecoin),
      "dogecoin-testnet" | "doge-testnet" => Ok(Self::DogecoinTestnet),
      "dogecoin-regtest" | "doge-regtest" => Ok(Self::DogecoinRegtest),
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
    assert_eq!("mainnet".parse::<Chain>().unwrap(), Chain::Mainnet);
    assert_eq!("regtest".parse::<Chain>().unwrap(), Chain::Regtest);
    assert_eq!("signet".parse::<Chain>().unwrap(), Chain::Signet);
    assert_eq!("testnet".parse::<Chain>().unwrap(), Chain::Testnet);
    assert_eq!("testnet4".parse::<Chain>().unwrap(), Chain::Testnet4);
    assert_eq!("dogecoin".parse::<Chain>().unwrap(), Chain::Dogecoin);
    assert_eq!("doge".parse::<Chain>().unwrap(), Chain::Dogecoin);
    assert_eq!(
      "dogecoin-testnet".parse::<Chain>().unwrap(),
      Chain::DogecoinTestnet
    );
    assert_eq!(
      "foo".parse::<Chain>().unwrap_err().to_string(),
      "Invalid chain `foo`"
    );
  }

  #[test]
  fn dogecoin_is_dogecoin() {
    assert!(Chain::Dogecoin.is_dogecoin());
    assert!(Chain::DogecoinTestnet.is_dogecoin());
    assert!(Chain::DogecoinRegtest.is_dogecoin());
    assert!(!Chain::Mainnet.is_dogecoin());
  }

  #[test]
  fn dogecoin_rpc_ports() {
    assert_eq!(Chain::Dogecoin.default_rpc_port(), 22555);
    assert_eq!(Chain::DogecoinTestnet.default_rpc_port(), 44555);
    assert_eq!(Chain::DogecoinRegtest.default_rpc_port(), 18444);
  }

  #[test]
  fn dogecoin_genesis_block() {
    // Just ensure it parses without panicking
    let _ = Chain::Dogecoin.genesis_block();
    let _ = Chain::DogecoinRegtest.genesis_block();
    let _ = Chain::DogecoinTestnet.genesis_block();
  }
}
