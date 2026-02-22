use {super::*, dog::subcommand::wallet::addresses::Output};

#[test]
fn addresses() {
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

  let output = CommandBuilder::new("--regtest --index-dunes wallet addresses")
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<BTreeMap<Address<NetworkUnchecked>, Vec<Output>>>();

  pretty_assert_eq!(
    output
      .get(&etched.output.dune.clone().unwrap().destination.unwrap())
      .unwrap(),
    &vec![Output {
      output: etched.output.dune.unwrap().location.unwrap(),
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
    }]
  );

  pretty_assert_eq!(
    output
      .get(&etched.output.inscriptions[0].destination)
      .unwrap(),
    &vec![Output {
      output: etched.output.inscriptions[0].location.outpoint,
      amount: 10000,
      inscriptions: Some(vec![etched.output.inscriptions[0].id]),
      dunes: Some(BTreeMap::new()),
    }]
  );
}
