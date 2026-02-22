use super::*;

#[derive(Debug, PartialEq, Clone, DeserializeFromStr, SerializeDisplay)]
pub enum Outgoing {
  Amount(Amount),
  InscriptionId(InscriptionId),
  Dune { decimal: Decimal, dune: SpacedDune },
  Koinu(Koinu),
  KoinuPoint(KoinuPoint),
}

impl Display for Outgoing {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Amount(amount) => write!(f, "{}", amount.to_string().to_lowercase()),
      Self::InscriptionId(inscription_id) => inscription_id.fmt(f),
      Self::Dune { decimal, dune } => write!(f, "{decimal}:{dune}"),
      Self::Koinu(sat) => write!(f, "{}", sat.name()),
      Self::KoinuPoint(satpoint) => satpoint.fmt(f),
    }
  }
}

impl FromStr for Outgoing {
  type Err = SnafuError;

  fn from_str(input: &str) -> Result<Self, Self::Err> {
    static AMOUNT: LazyLock<Regex> = LazyLock::new(|| {
      Regex::new(
        r"(?x)
        ^
        (
          \d+
          |
          \.\d+
          |
          \d+\.\d+
        )
        \ ?
        (bit|btc|cbtc|mbtc|msat|nbtc|pbtc|sat|koinu|ubtc)
        (s)?
        $
        ",
      )
      .unwrap()
    });

    static RUNE: LazyLock<Regex> = LazyLock::new(|| {
      Regex::new(
        r"(?x)
        ^
        (
          \d+
          |
          \.\d+
          |
          \d+\.\d+
        )
        \s*:\s*
        (
          [A-Z•.]+
        )
        $
        ",
      )
      .unwrap()
    });

    if re::SAT_NAME.is_match(input) {
      Ok(Outgoing::Koinu(
        input.parse().snafu_context(error::SatParse { input })?,
      ))
    } else if re::SATPOINT.is_match(input) {
      Ok(Outgoing::KoinuPoint(
        input
          .parse()
          .snafu_context(error::KoinuPointParse { input })?,
      ))
    } else if re::INSCRIPTION_ID.is_match(input) {
      Ok(Outgoing::InscriptionId(
        input
          .parse()
          .snafu_context(error::InscriptionIdParse { input })?,
      ))
    } else if AMOUNT.is_match(input) {
      Ok(Outgoing::Amount(
        input.parse().snafu_context(error::AmountParse { input })?,
      ))
    } else if let Some(captures) = RUNE.captures(input) {
      let decimal = captures[1]
        .parse::<Decimal>()
        .snafu_context(error::RuneAmountParse { input })?;
      let dune = captures[2]
        .parse()
        .snafu_context(error::RuneParse { input })?;
      Ok(Self::Dune { decimal, dune })
    } else {
      Err(SnafuError::OutgoingParse {
        input: input.to_string(),
      })
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_str() {
    #[track_caller]
    fn case(s: &str, outgoing: Outgoing) {
      assert_eq!(s.parse::<Outgoing>().unwrap(), outgoing);
    }

    case("nvtdijuwxlp", Outgoing::Koinu("nvtdijuwxlp".parse().unwrap()));
    case("a", Outgoing::Koinu("a".parse().unwrap()));

    case(
      "0000000000000000000000000000000000000000000000000000000000000000i0",
      Outgoing::InscriptionId(
        "0000000000000000000000000000000000000000000000000000000000000000i0"
          .parse()
          .unwrap(),
      ),
    );

    case(
      "0000000000000000000000000000000000000000000000000000000000000000:0:0",
      Outgoing::KoinuPoint(
        "0000000000000000000000000000000000000000000000000000000000000000:0:0"
          .parse()
          .unwrap(),
      ),
    );

    case("0 btc", Outgoing::Amount("0 btc".parse().unwrap()));
    case("0btc", Outgoing::Amount("0 btc".parse().unwrap()));
    case("0.0btc", Outgoing::Amount("0 btc".parse().unwrap()));
    case(".0btc", Outgoing::Amount("0 btc".parse().unwrap()));

    case(
      "0  : XYZ",
      Outgoing::Dune {
        dune: "XYZ".parse().unwrap(),
        decimal: "0".parse().unwrap(),
      },
    );

    case(
      "0:XYZ",
      Outgoing::Dune {
        dune: "XYZ".parse().unwrap(),
        decimal: "0".parse().unwrap(),
      },
    );

    case(
      "0.0:XYZ",
      Outgoing::Dune {
        dune: "XYZ".parse().unwrap(),
        decimal: "0.0".parse().unwrap(),
      },
    );

    case(
      ".0:XYZ",
      Outgoing::Dune {
        dune: "XYZ".parse().unwrap(),
        decimal: ".0".parse().unwrap(),
      },
    );

    case(
      "1.1:XYZ",
      Outgoing::Dune {
        dune: "XYZ".parse().unwrap(),
        decimal: "1.1".parse().unwrap(),
      },
    );

    case(
      "1.1:X.Y.Z",
      Outgoing::Dune {
        dune: "X.Y.Z".parse().unwrap(),
        decimal: "1.1".parse().unwrap(),
      },
    );
  }

  #[test]
  fn roundtrip() {
    #[track_caller]
    fn case(s: &str, outgoing: Outgoing) {
      assert_eq!(s.parse::<Outgoing>().unwrap(), outgoing);
      assert_eq!(s, outgoing.to_string());
    }

    case("nvtdijuwxlp", Outgoing::Koinu("nvtdijuwxlp".parse().unwrap()));
    case("a", Outgoing::Koinu("a".parse().unwrap()));

    case(
      "0000000000000000000000000000000000000000000000000000000000000000i0",
      Outgoing::InscriptionId(
        "0000000000000000000000000000000000000000000000000000000000000000i0"
          .parse()
          .unwrap(),
      ),
    );

    case(
      "0000000000000000000000000000000000000000000000000000000000000000:0:0",
      Outgoing::KoinuPoint(
        "0000000000000000000000000000000000000000000000000000000000000000:0:0"
          .parse()
          .unwrap(),
      ),
    );

    case("0 btc", Outgoing::Amount("0 btc".parse().unwrap()));
    case(
      "1.20000000 btc",
      Outgoing::Amount("1.2 btc".parse().unwrap()),
    );

    case(
      "0:XY•Z",
      Outgoing::Dune {
        dune: "XY•Z".parse().unwrap(),
        decimal: "0".parse().unwrap(),
      },
    );

    case(
      "1.1:XYZ",
      Outgoing::Dune {
        dune: "XYZ".parse().unwrap(),
        decimal: "1.1".parse().unwrap(),
      },
    );
  }

  #[test]
  fn serde() {
    #[track_caller]
    fn case(s: &str, j: &str, o: Outgoing) {
      assert_eq!(s.parse::<Outgoing>().unwrap(), o);
      assert_eq!(serde_json::to_string(&o).unwrap(), j);
      assert_eq!(serde_json::from_str::<Outgoing>(j).unwrap(), o);
    }

    case(
      "nvtdijuwxlp",
      "\"nvtdijuwxlp\"",
      Outgoing::Koinu("nvtdijuwxlp".parse().unwrap()),
    );
    case("a", "\"a\"", Outgoing::Koinu("a".parse().unwrap()));

    case(
      "0000000000000000000000000000000000000000000000000000000000000000i0",
      "\"0000000000000000000000000000000000000000000000000000000000000000i0\"",
      Outgoing::InscriptionId(
        "0000000000000000000000000000000000000000000000000000000000000000i0"
          .parse()
          .unwrap(),
      ),
    );

    case(
      "0000000000000000000000000000000000000000000000000000000000000000:0:0",
      "\"0000000000000000000000000000000000000000000000000000000000000000:0:0\"",
      Outgoing::KoinuPoint(
        "0000000000000000000000000000000000000000000000000000000000000000:0:0"
          .parse()
          .unwrap(),
      ),
    );

    case(
      "3 btc",
      "\"3 btc\"",
      Outgoing::Amount(Amount::from_sat(3 * COIN_VALUE)),
    );

    case(
      "6.66:HELL.MONEY",
      "\"6.66:HELL•MONEY\"",
      Outgoing::Dune {
        dune: "HELL•MONEY".parse().unwrap(),
        decimal: "6.66".parse().unwrap(),
      },
    );
  }
}
