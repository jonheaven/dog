use {
  super::*,
  dog::{decimal::Decimal, subcommand::wallet::runics::RunicUtxo},
};

#[test]
fn wallet_runics() {
  let core = mockcore::builder().network(Network::Regtest).build();
  let dog = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-dunes"], &[]);

  create_wallet(&core, &dog);

  let dune = Dune(RUNE);

  batch(
    &core,
    &dog,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        premine: "1000".parse().unwrap(),
        dune: SpacedDune { dune, spacers: 1 },
        supply: "1000".parse().unwrap(),
        symbol: '¢',
        terms: None,
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  pretty_assert_eq!(
    CommandBuilder::new("--regtest --index-dunes wallet runics")
      .core(&core)
      .dog(&dog)
      .run_and_deserialize_output::<Vec<RunicUtxo>>()
      .first()
      .unwrap()
      .dunes,
    vec![(
      SpacedDune { dune, spacers: 1 },
      Decimal {
        value: 1000,
        scale: 0
      }
    )]
    .into_iter()
    .collect()
  );
}
