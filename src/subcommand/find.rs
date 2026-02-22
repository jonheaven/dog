use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Find {
  #[arg(help = "Find output and offset of <SAT>.")]
  sat: Koinu,
  #[clap(help = "Find output and offset of all koinu in the range [<SAT>, <END>).")]
  end: Option<Koinu>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub satpoint: KoinuPoint,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FindRangeOutput {
  pub start: u64,
  pub size: u64,
  pub satpoint: KoinuPoint,
}

impl Find {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;

    if !index.has_sat_index() {
      bail!("find requires index created with `--index-koinu` flag");
    }

    index.update()?;

    match self.end {
      Some(end) => match index.find_range(self.sat, end)? {
        Some(mut results) => {
          results.sort_by_key(|find_range_output| find_range_output.start);
          Ok(Some(Box::new(results)))
        }
        None => Err(anyhow!("range has not been mined as of index height")),
      },
      None => match index.find(self.sat)? {
        Some(satpoint) => Ok(Some(Box::new(Output { satpoint }))),
        None => Err(anyhow!("sat has not been mined as of index height")),
      },
    }
  }
}
