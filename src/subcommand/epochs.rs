use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub starting_koinu: Vec<Koinu>,
}

pub(crate) fn run() -> SubcommandResult {
  let mut starting_koinu = Vec::new();
  for sat in Epoch::all_starting_koinu() {
    starting_koinu.push(sat);
  }

  Ok(Some(Box::new(Output { starting_koinu })))
}
