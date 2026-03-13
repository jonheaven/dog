use super::*;

#[derive(Boilerplate, Debug, PartialEq)]
pub(crate) struct KoinuRelicsHtml;

impl PageContent for KoinuRelicsHtml {
  fn title(&self) -> String {
    "Koinu Relics".to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_regex_match!(
      KoinuRelicsHtml.to_string(),
      r"(?s).*<h1>Koinu Relics</h1>.*<a href=/static/koinu-relic-auto-theme.html>Open Preview Template</a>.*"
    );
  }
}
