use {super::*, dog::subcommand::dunes::Output};

#[test]
fn flag_is_required() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let dog = TestServer::spawn_with_server_args(&core, &["--regtest"], &[]);

  CommandBuilder::new("--regtest dunes")
    .core(&core)
    .dog(&dog)
    .expected_exit_code(1)
    .expected_stderr("error: `dog dunes` requires index created with `--index-dunes` flag\n")
    .run_and_extract_stdout();
}

#[test]
fn no_runes() {
  let core = mockcore::builder().network(Network::Regtest).build();

  assert_eq!(
    CommandBuilder::new("--index-dunes --regtest dunes")
      .core(&core)
      .run_and_deserialize_output::<Output>(),
    Output {
      dunes: BTreeMap::new(),
    }
  );
}

#[test]
fn one_rune() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let dog = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-dunes"], &[]);

  create_wallet(&core, &dog);

  let etch = etch(&core, &dog, Dune(RUNE));

  pretty_assert_eq!(
    CommandBuilder::new("--index-dunes --regtest dunes")
      .core(&core)
      .run_and_deserialize_output::<Output>(),
    Output {
      dunes: vec![(
        Dune(RUNE),
        RuneInfo {
          block: 7,
          burned: 0,
          divisibility: 0,
          etching: etch.output.reveal,
          id: DuneId { block: 7, tx: 1 },
          terms: None,
          mints: 0,
          number: 0,
          premine: 1000,
          dune: SpacedDune {
            dune: Dune(RUNE),
            spacers: 0
          },
          supply: 1000,
          symbol: Some('¢'),
          timestamp: dog::timestamp(7),
          turbo: false,
          tx: 1,
        }
      )]
      .into_iter()
      .collect(),
    }
  );
}

#[test]
fn two_runes() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let dog = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-dunes"], &[]);

  create_wallet(&core, &dog);

  let a = etch(&core, &dog, Dune(RUNE));
  let b = etch(&core, &dog, Dune(RUNE + 1));

  pretty_assert_eq!(
    CommandBuilder::new("--index-dunes --regtest dunes")
      .core(&core)
      .run_and_deserialize_output::<Output>(),
    Output {
      dunes: vec![
        (
          Dune(RUNE),
          RuneInfo {
            block: 7,
            burned: 0,
            divisibility: 0,
            etching: a.output.reveal,
            id: DuneId { block: 7, tx: 1 },
            terms: None,
            mints: 0,
            number: 0,
            premine: 1000,
            dune: SpacedDune {
              dune: Dune(RUNE),
              spacers: 0
            },
            supply: 1000,
            symbol: Some('¢'),
            timestamp: dog::timestamp(7),
            turbo: false,
            tx: 1,
          }
        ),
        (
          Dune(RUNE + 1),
          RuneInfo {
            block: 14,
            burned: 0,
            divisibility: 0,
            etching: b.output.reveal,
            id: DuneId { block: 14, tx: 1 },
            terms: None,
            mints: 0,
            number: 1,
            premine: 1000,
            dune: SpacedDune {
              dune: Dune(RUNE + 1),
              spacers: 0
            },
            supply: 1000,
            symbol: Some('¢'),
            timestamp: dog::timestamp(14),
            turbo: false,
            tx: 1,
          }
        )
      ]
      .into_iter()
      .collect(),
    }
  );
}
