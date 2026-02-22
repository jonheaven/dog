use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub dunes: BTreeMap<Dune, RuneInfo>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneInfo {
  pub block: u64,
  pub burned: u128,
  pub divisibility: u8,
  pub etching: Txid,
  pub id: DuneId,
  pub mints: u128,
  pub number: u64,
  pub premine: u128,
  pub dune: SpacedDune,
  pub supply: u128,
  pub symbol: Option<char>,
  pub terms: Option<Terms>,
  pub timestamp: DateTime<Utc>,
  pub turbo: bool,
  pub tx: u32,
}

pub(crate) fn run(settings: Settings) -> SubcommandResult {
  let index = Index::open(&settings)?;

  ensure!(
    index.has_rune_index(),
    "`ord dunes` requires index created with `--index-dunes` flag",
  );

  index.update()?;

  Ok(Some(Box::new(Output {
    dunes: index
      .dunes()?
      .into_iter()
      .map(
        |(
          id,
          entry @ DuneEntry {
            block,
            burned,
            divisibility,
            etching,
            mints,
            number,
            premine,
            spaced_dune,
            symbol,
            terms,
            timestamp,
            turbo,
          },
        )| {
          (
            spaced_dune.dune,
            RuneInfo {
              block,
              burned,
              divisibility,
              etching,
              id,
              mints,
              number,
              premine,
              dune: spaced_dune,
              supply: entry.supply(),
              symbol,
              terms,
              timestamp: crate::timestamp(timestamp),
              turbo,
              tx: id.tx,
            },
          )
        },
      )
      .collect::<BTreeMap<Dune, RuneInfo>>(),
  })))
}
