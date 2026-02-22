use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SplitfileUnchecked {
  outputs: Vec<OutputUnchecked>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OutputUnchecked {
  address: Address<NetworkUnchecked>,
  value: Option<DeserializeFromStr<Amount>>,
  dunes: BTreeMap<SpacedDune, Decimal>,
}

pub(crate) struct Splitfile {
  pub(crate) outputs: Vec<Output>,
  pub(crate) rune_info: BTreeMap<Dune, RuneInfo>,
}

pub(crate) struct Output {
  pub(crate) address: Address,
  pub(crate) value: Option<Amount>,
  pub(crate) dunes: BTreeMap<Dune, u128>,
}

#[derive(Clone, Copy)]
pub(crate) struct RuneInfo {
  pub(crate) divisibility: u8,
  pub(crate) id: DuneId,
  pub(crate) spaced_dune: SpacedDune,
  pub(crate) symbol: Option<char>,
}

impl Splitfile {
  pub(crate) fn load(path: &Path, wallet: &Wallet) -> Result<Self> {
    let network = wallet.chain().network();

    let unchecked = Self::load_unchecked(path)?;

    let mut rune_info = BTreeMap::<Dune, RuneInfo>::new();

    let mut outputs = Vec::new();

    for output in unchecked.outputs {
      let mut dunes = BTreeMap::new();

      for (spaced_dune, decimal) in output.dunes {
        let info = if let Some(info) = rune_info.get(&spaced_dune.dune) {
          info
        } else {
          let (id, entry, _parent) = wallet
            .get_rune(spaced_dune.dune)?
            .with_context(|| format!("dune `{}` has not been etched", spaced_dune.dune))?;
          rune_info.insert(
            spaced_dune.dune,
            RuneInfo {
              divisibility: entry.divisibility,
              id,
              spaced_dune: entry.spaced_dune,
              symbol: entry.symbol,
            },
          );
          rune_info.get(&spaced_dune.dune).unwrap()
        };

        let amount = decimal.to_integer(info.divisibility)?;

        dunes.insert(spaced_dune.dune, amount);
      }

      outputs.push(Output {
        address: output.address.require_network(network)?,
        value: output.value.map(|DeserializeFromStr(value)| value),
        dunes,
      });
    }

    Ok(Self { outputs, rune_info })
  }

  fn load_unchecked(path: &Path) -> Result<SplitfileUnchecked> {
    Ok(serde_yaml::from_reader(File::open(path)?)?)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn example_split_file_is_valid() {
    Splitfile::load_unchecked("splits.yaml".as_ref()).unwrap();
  }
}
