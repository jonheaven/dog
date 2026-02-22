use super::*;

#[test]
fn flag_is_required() {
  let core = mockcore::builder().network(Network::Regtest).build();

  CommandBuilder::new("--regtest balances")
    .core(&core)
    .expected_exit_code(1)
    .expected_stderr("error: `dog balances` requires index created with `--index-dunes` flag\n")
    .run_and_extract_stdout();
}

#[test]
fn no_runes() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let output = CommandBuilder::new("--regtest --index-dunes balances")
    .core(&core)
    .run_and_deserialize_output::<Balances>();

  assert_eq!(
    output,
    Balances {
      dunes: BTreeMap::new()
    }
  );
}

#[test]
fn with_runes() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let dog = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-dunes"], &[]);

  create_wallet(&core, &dog);

  let a = etch(&core, &dog, Dune(RUNE));
  let b = etch(&core, &dog, Dune(RUNE + 1));

  let output = CommandBuilder::new("--regtest --index-dunes balances")
    .core(&core)
    .run_and_deserialize_output::<Balances>();

  assert_eq!(
    output,
    Balances {
      dunes: [
        (
          SpacedDune::new(Dune(RUNE), 0),
          [(
            OutPoint {
              txid: a.output.reveal,
              vout: 1
            },
            Pile {
              amount: 1000,
              divisibility: 0,
              symbol: Some('¢')
            },
          )]
          .into()
        ),
        (
          SpacedDune::new(Dune(RUNE + 1), 0),
          [(
            OutPoint {
              txid: b.output.reveal,
              vout: 1
            },
            Pile {
              amount: 1000,
              divisibility: 0,
              symbol: Some('¢')
            },
          )]
          .into()
        ),
      ]
      .into()
    }
  );
}
