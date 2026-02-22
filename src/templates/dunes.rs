use super::*;

#[derive(Boilerplate, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunesHtml {
  pub entries: Vec<(DuneId, DuneEntry)>,
  pub more: bool,
  pub prev: Option<usize>,
  pub next: Option<usize>,
}

impl PageContent for RunesHtml {
  fn title(&self) -> String {
    "Runes".to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(
      RunesHtml {
        entries: vec![(
          DuneId { block: 0, tx: 0 },
          DuneEntry {
            spaced_dune: SpacedDune {
              dune: Dune(26),
              spacers: 1
            },
            ..default()
          }
        )],
        more: false,
        prev: None,
        next: None,
      }
      .to_string(),
      "<h1>Runes</h1>
<ul>
  <li><a href=/dune/A•A>A•A</a></li>
</ul>
<div class=center>
    prev
      next
  </div>"
    );
  }

  #[test]
  fn with_prev_and_next() {
    assert_eq!(
      RunesHtml {
        entries: vec![
          (
            DuneId { block: 0, tx: 0 },
            DuneEntry {
              spaced_dune: SpacedDune {
                dune: Dune(0),
                spacers: 0
              },
              ..Default::default()
            }
          ),
          (
            DuneId { block: 0, tx: 1 },
            DuneEntry {
              spaced_dune: SpacedDune {
                dune: Dune(2),
                spacers: 0
              },
              ..Default::default()
            }
          )
        ],
        prev: Some(1),
        next: Some(2),
        more: true,
      }
      .to_string(),
      "<h1>Runes</h1>
<ul>
  <li><a href=/dune/A>A</a></li>
  <li><a href=/dune/C>C</a></li>
</ul>
<div class=center>
    <a class=prev href=/dunes/1>prev</a>
      <a class=next href=/dunes/2>next</a>
  </div>"
    );
  }
}
