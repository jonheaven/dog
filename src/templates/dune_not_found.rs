use super::*;

#[derive(Boilerplate, Debug, PartialEq, Serialize)]
pub struct RuneNotFoundHtml {
  pub dune: Dune,
  pub unlock: Option<(Height, Blocktime)>,
}

impl PageContent for RuneNotFoundHtml {
  fn title(&self) -> String {
    format!("Dune {}", self.dune)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display_expected() {
    assert_regex_match!(
      RuneNotFoundHtml {
        dune: Dune(u128::MAX),
        unlock: Some((Height(111), Blocktime::Expected(DateTime::default()))),
      },
      r"<h1>BCGDENLQRQWDSLRUGSNLBTMFIJAV</h1>
<dl>
  <dt>unlock height</dt>
  <dd>111</dd>
  <dt>unlock time</dt>
  <dd><time>1970-01-01 00:00:00 UTC</time> \(expected\)</dd>
  <dt>reserved</dt>
  <dd>false</dd>
</dl>
"
    );
  }

  #[test]
  fn display_confirmed() {
    assert_regex_match!(
      RuneNotFoundHtml {
        dune: Dune(u128::MAX),
        unlock: Some((Height(111), Blocktime::Confirmed(DateTime::default()))),
      },
      r"<h1>BCGDENLQRQWDSLRUGSNLBTMFIJAV</h1>
<dl>
  <dt>unlock height</dt>
  <dd>111</dd>
  <dt>unlock time</dt>
  <dd><time>1970-01-01 00:00:00 UTC</time></dd>
  <dt>reserved</dt>
  <dd>false</dd>
</dl>
"
    );
  }

  #[test]
  fn display_reserved() {
    assert_regex_match!(
      RuneNotFoundHtml {
        dune: Dune(Dune::RESERVED),
        unlock: None,
      },
      "<h1>AAAAAAAAAAAAAAAAAAAAAAAAAAA</h1>
<dl>
  <dt>unlock height</dt>
  <dd>none</dd>
  <dt>reserved</dt>
  <dd>true</dd>
</dl>
"
    );
  }
}
