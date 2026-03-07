use super::*;

#[derive(Boilerplate)]
pub(crate) struct AddressHtml {
  pub(crate) address: String,
  pub(crate) header: bool,
  pub(crate) inscriptions: Option<Vec<InscriptionId>>,
  pub(crate) outputs: Vec<OutPoint>,
  pub(crate) dunes_balances: Option<Vec<(SpacedDune, Decimal, Option<char>)>>,
  pub(crate) sat_balance: u64,
  pub(crate) lazy_lookup: bool,
}

impl PageContent for AddressHtml {
  fn title(&self) -> String {
    format!("Address {}", self.address)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn setup() -> AddressHtml {
    AddressHtml {
      address: "DHrqn6H6ocgbRB1Szu7Q1sn1tVTfkpinnc".to_string(),
      header: true,
      outputs: vec![outpoint(1), outpoint(2)],
      inscriptions: Some(vec![inscription_id(1)]),
      sat_balance: 99,
      lazy_lookup: false,
      dunes_balances: Some(vec![
        (
          SpacedDune {
            dune: Dune::from_str("TEEEEEEEEESTDUNE").unwrap(),
            spacers: 0,
          },
          Decimal {
            scale: 0,
            value: 20000,
          },
          Some('R'),
        ),
        (
          SpacedDune {
            dune: Dune::from_str("ANOTHERTEESTDUNE").unwrap(),
            spacers: 0,
          },
          Decimal {
            scale: 0,
            value: 10000,
          },
          Some('F'),
        ),
      ]),
    }
  }

  #[test]
  fn test_address_rendering() {
    let address_html = setup();
    let expected_pattern =
      r#".*<h1>Address DHrqn6H6ocgbRB1Szu7Q1sn1tVTfkpinnc</h1>.*"#;
    assert_regex_match!(address_html, expected_pattern);
  }

  #[test]
  fn test_sat_balance_rendering() {
    let address_html = setup();
    let expected_pattern = r#".*<dt>koinu balance</dt>\n\s*<dd>99</dd>.*"#;
    assert_regex_match!(address_html, expected_pattern);
  }

  #[test]
  fn test_inscriptions_rendering() {
    let address_html = setup();
    let expected_pattern = r#".*<dt>inscriptions</dt>\n\s*<dd class=thumbnails>.*<a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1></iframe></a>.*</dd>.*"#;
    assert_regex_match!(address_html, expected_pattern);
  }

  #[test]
  fn test_dunes_balances_rendering() {
    let address_html = setup();
    let expected_pattern = r#".*<dt>dune balances</dt>\n\s*<dd><a class=monospace href=/dune/TEEEEEEEEESTDUNE>TEEEEEEEEESTDUNE</a>: 20000R</dd>\n\s*<dd><a class=monospace href=/dune/ANOTHERTEESTDUNE>ANOTHERTEESTDUNE</a>: 10000F</dd>.*"#;
    assert_regex_match!(address_html, expected_pattern);
  }

  #[test]
  fn test_outputs_rendering() {
    let address_html = setup();
    let expected_pattern = r#".*<dt>outputs</dt>\n\s*<dd>\n\s*<ul>\n\s*<li><a class=collapse href=/output/1{64}:1>1{64}:1</a></li>\n\s*<li><a class=collapse href=/output/2{64}:2>2{64}:2</a></li>\n\s*</ul>\n\s*</dd>.*"#;
    assert_regex_match!(address_html, expected_pattern);
  }
}
