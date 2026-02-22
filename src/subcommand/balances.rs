use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub dunes: BTreeMap<SpacedDune, BTreeMap<OutPoint, Pile>>,
}

pub(crate) fn run(settings: Settings) -> SubcommandResult {
  let index = Index::open(&settings)?;

  ensure!(
    index.has_rune_index(),
    "`ord balances` requires index created with `--index-dunes` flag",
  );

  index.update()?;

  Ok(Some(Box::new(Output {
    dunes: index.get_rune_balance_map()?,
  })))
}
