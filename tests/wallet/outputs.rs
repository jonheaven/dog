use {super::*, dog::subcommand::wallet::outputs::Output};

#[test]
fn outputs() {
  let core = mockcore::spawn();

  let dog = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &dog);

  let coinbase_tx = &core.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.compute_txid(), 0);
  let amount = coinbase_tx.output[0].value;

  let output = CommandBuilder::new("wallet outputs")
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<Vec<Output>>();

  assert_eq!(output[0].output, outpoint);
  assert_eq!(output[0].amount, amount.to_sat());
  assert!(output[0].koinu_ranges.is_none());
}

#[test]
fn outputs_includes_locked_outputs() {
  let core = mockcore::spawn();

  let dog = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &dog);

  let coinbase_tx = &core.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.compute_txid(), 0);
  let amount = coinbase_tx.output[0].value;

  core.lock(outpoint);

  let output = CommandBuilder::new("wallet outputs")
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<Vec<Output>>();

  assert_eq!(output[0].output, outpoint);
  assert_eq!(output[0].amount, amount.to_sat());
  assert!(output[0].koinu_ranges.is_none());
}

#[test]
fn outputs_includes_unbound_outputs() {
  let core = mockcore::spawn();

  let dog = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &dog);

  let coinbase_tx = &core.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.compute_txid(), 0);
  let amount = coinbase_tx.output[0].value;

  core.lock(outpoint);

  let output = CommandBuilder::new("wallet outputs")
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<Vec<Output>>();

  assert_eq!(output[0].output, outpoint);
  assert_eq!(output[0].amount, amount.to_sat());
  assert!(output[0].koinu_ranges.is_none());
}

#[test]
fn outputs_includes_sat_ranges() {
  let core = mockcore::spawn();

  let dog = TestServer::spawn_with_server_args(&core, &["--index-koinu"], &[]);

  create_wallet(&core, &dog);

  let coinbase_tx = &core.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.compute_txid(), 0);
  let amount = coinbase_tx.output[0].value;

  let output = CommandBuilder::new("wallet outputs --ranges")
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<Vec<Output>>();

  assert_eq!(output[0].output, outpoint);
  assert_eq!(output[0].amount, amount.to_sat());
  assert_eq!(
    output[0].koinu_ranges,
    Some(vec!["5000000000-5001000000".to_string()])
  );
}

#[test]
fn outputs_includes_runes_and_inscriptions() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let dog = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-dunes"], &[]);

  create_wallet(&core, &dog);

  let dune = Dune(RUNE);

  let etched = batch(
    &core,
    &dog,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 3,
        premine: "1.111".parse().unwrap(),
        dune: SpacedDune { dune, spacers: 1 },
        supply: "2.222".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          amount: "1.111".parse().unwrap(),
          cap: 1,
          ..default()
        }),
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  let output = CommandBuilder::new("--regtest --index-dunes wallet outputs")
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<Vec<Output>>();

  assert!(
    output.contains(&Output {
      output: etched.output.dune.clone().unwrap().location.unwrap(),
      address: etched.output.dune.unwrap().destination,
      amount: 10000,
      inscriptions: Some(Vec::new()),
      dunes: Some(
        vec![(
          SpacedDune { dune, spacers: 1 },
          dog::decimal::Decimal {
            value: 1111,
            scale: 3,
          }
        )]
        .into_iter()
        .collect()
      ),
      koinu_ranges: None,
    })
  );

  assert!(output.contains(&Output {
    output: etched.output.inscriptions[0].location.outpoint,
    address: Some(etched.output.inscriptions[0].destination.clone()),
    amount: 10000,
    inscriptions: Some(vec![etched.output.inscriptions[0].id]),
    dunes: Some(BTreeMap::new()),
    koinu_ranges: None,
  }));
}
