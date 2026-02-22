use {
  super::*,
  dog::subcommand::wallet::{cardinals::CardinalUtxo, outputs::Output},
};

#[test]
fn cardinals() {
  let core = mockcore::spawn();

  let dog = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &dog);

  inscribe(&core, &dog);

  let all_outputs = CommandBuilder::new("wallet outputs")
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<Vec<Output>>();

  let cardinal_outputs = CommandBuilder::new("wallet cardinals")
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<Vec<CardinalUtxo>>();

  assert_eq!(all_outputs.len() - cardinal_outputs.len(), 1);
}

#[test]
fn cardinals_does_not_show_runic_outputs() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let dog = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-dunes"], &[]);

  create_wallet(&core, &dog);

  core.mine_blocks(1);

  batch(
    &core,
    &dog,
    batch::File {
      etching: Some(batch::Etching {
        supply: "1000".parse().unwrap(),
        divisibility: 0,
        terms: None,
        premine: "1000".parse().unwrap(),
        dune: SpacedDune {
          dune: Dune(RUNE),
          spacers: 0,
        },
        symbol: '¢',
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  let all_outputs = CommandBuilder::new("--regtest wallet outputs")
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<Vec<Output>>();

  let cardinal_outputs = CommandBuilder::new("--regtest wallet cardinals")
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<Vec<CardinalUtxo>>();

  assert_eq!(all_outputs.len() - cardinal_outputs.len(), 2);
}
