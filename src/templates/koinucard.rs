use super::*;

#[derive(Boilerplate)]
pub(crate) struct KoinucardHtml {
  pub(crate) koinucard: Option<(Koinucard, Option<AddressHtml>)>,
}

impl PageContent for KoinucardHtml {
  fn title(&self) -> String {
    if let Some((koinucard, _address_info)) = &self.koinucard {
      format!("Koinucard {}", koinucard.address)
    } else {
      "Koinucard".into()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn title() {
    assert_eq!(
      KoinucardHtml {
        koinucard: Some((crate::koinucard::tests::coinkite_koinucard(), None)),
      }
      .title(),
      format!("Koinucard {}", crate::koinucard::tests::coinkite_address())
    );

    assert_eq!(KoinucardHtml { koinucard: None }.title(), "Koinucard");
  }

  #[test]
  fn no_address_info() {
    pretty_assert_eq!(
      KoinucardHtml {
        koinucard: Some((crate::koinucard::tests::coinkite_koinucard(), None)),
      }
      .to_string(),
      r#"<h1>Koinucard bc1ql86vqdwylsgmgkkrae5nrafte8yp43a5x2tplf</h1>
<form>
  <label for=url>Koinucard URL</label>
  <input
    type=text
    id=url
    name=url
    required
  >
  <input type="submit" value="Submit">
</form>
<dl>
  <dt>slot</dt>
  <dd>1</dd>
  <dt>state</dt>
  <dd class=koinucard-sealed>sealed</dd>
  <dt>address</dt>
  <dd><a class=collapse href=/address/bc1ql86vqdwylsgmgkkrae5nrafte8yp43a5x2tplf>bc1ql86vqdwylsgmgkkrae5nrafte8yp43a5x2tplf</a></dd>
  <dt>nonce</dt>
  <dd>7664168a4ef7b8e8</dd>
</dl>
"#,
    );
  }

  #[test]
  fn with_address_info() {
    pretty_assert_eq!(
      KoinucardHtml {
        koinucard: Some((
          crate::koinucard::tests::coinkite_koinucard(),
          Some(AddressHtml {
            address: crate::koinucard::tests::coinkite_address().to_string(),
            header: false,
            inscriptions: Some(Vec::new()),
            outputs: Vec::new(),
            dunes_balances: None,
            sat_balance: 0,
            lazy_lookup: false,
          })
        )),
      }
      .to_string(),
      r#"<h1>Koinucard bc1ql86vqdwylsgmgkkrae5nrafte8yp43a5x2tplf</h1>
<form>
  <label for=url>Koinucard URL</label>
  <input
    type=text
    id=url
    name=url
    required
  >
  <input type="submit" value="Submit">
</form>
<dl>
  <dt>slot</dt>
  <dd>1</dd>
  <dt>state</dt>
  <dd class=koinucard-sealed>sealed</dd>
  <dt>address</dt>
  <dd><a class=collapse href=/address/bc1ql86vqdwylsgmgkkrae5nrafte8yp43a5x2tplf>bc1ql86vqdwylsgmgkkrae5nrafte8yp43a5x2tplf</a></dd>
  <dt>nonce</dt>
  <dd>7664168a4ef7b8e8</dd>
</dl>
<dl>
  <dt>koinu balance</dt>
  <dd>0</dd>
  <dt>outputs</dt>
  <dd>
    <ul>
    </ul>
  </dd>
</dl>

"#,
    );
  }

  #[test]
  fn state_error() {
    assert_regex_match! {
      KoinucardHtml {
        koinucard: Some((
          Koinucard {
            state: crate::koinucard::State::Error,
            ..crate::koinucard::tests::coinkite_koinucard()
          },
          Some(AddressHtml {
            address: crate::koinucard::tests::coinkite_address().to_string(),
            header: false,
            inscriptions: Some(Vec::new()),
            outputs: Vec::new(),
            dunes_balances: None,
            sat_balance: 0,
            lazy_lookup: false,
          })
        )),
      }
      .to_string(),
      r#".*
  <dt>state</dt>
  <dd class=koinucard-error>error</dd>
.*
"#,
    }
  }

  #[test]
  fn state_unsealed() {
    assert_regex_match! {
      KoinucardHtml {
        koinucard: Some((
          Koinucard {
            state: crate::koinucard::State::Unsealed,
            ..crate::koinucard::tests::coinkite_koinucard()
          },
          Some(AddressHtml {
            address: crate::koinucard::tests::coinkite_address().to_string(),
            header: false,
            inscriptions: Some(Vec::new()),
            outputs: Vec::new(),
            dunes_balances: None,
            sat_balance: 0,
            lazy_lookup: false,
          })
        )),
      }
      .to_string(),
      r#".*
  <dt>state</dt>
  <dd class=koinucard-unsealed>unsealed</dd>
.*
"#,
    }
  }
}
