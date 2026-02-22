use {super::*, dog::decimal::Decimal};

#[test]
fn wallet_balance() {
  let core = mockcore::spawn();

  let dog = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &dog);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .core(&core)
      .dog(&dog)
      .run_and_deserialize_output::<Balance>()
      .cardinal,
    0
  );

  core.mine_blocks(1);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .core(&core)
      .dog(&dog)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 50 * COIN_VALUE,
      ordinal: 0,
      runic: None,
      dunes: None,
      total: 50 * COIN_VALUE,
    }
  );
}

#[test]
fn inscribed_utxos_are_deducted_from_cardinal() {
  let core = mockcore::spawn();

  let dog = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &dog);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .core(&core)
      .dog(&dog)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 0,
      ordinal: 0,
      runic: None,
      dunes: None,
      total: 0,
    }
  );

  inscribe(&core, &dog);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .core(&core)
      .dog(&dog)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 100 * COIN_VALUE - 10_000,
      ordinal: 10_000,
      runic: None,
      dunes: None,
      total: 100 * COIN_VALUE,
    }
  );
}

#[test]
fn runic_utxos_are_deducted_from_cardinal() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let dog = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-dunes"], &[]);

  create_wallet(&core, &dog);

  pretty_assert_eq!(
    CommandBuilder::new("--regtest --index-dunes wallet balance")
      .core(&core)
      .dog(&dog)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 0,
      ordinal: 0,
      runic: Some(0),
      dunes: Some(BTreeMap::new()),
      total: 0,
    }
  );

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
    CommandBuilder::new("--regtest --index-dunes wallet balance")
      .core(&core)
      .dog(&dog)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 50 * COIN_VALUE * 7 - 20_000,
      ordinal: 10000,
      runic: Some(10_000),
      dunes: Some(
        vec![(
          SpacedDune { dune, spacers: 1 },
          Decimal {
            value: 1000,
            scale: 0,
          }
        )]
        .into_iter()
        .collect()
      ),
      total: 50 * COIN_VALUE * 7,
    }
  );
}

#[test]
fn unsynced_wallet_fails_with_unindexed_output() {
  let core = mockcore::spawn();
  let dog = TestServer::spawn(&core);

  core.mine_blocks(1);

  create_wallet(&core, &dog);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .dog(&dog)
      .core(&core)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 50 * COIN_VALUE,
      ordinal: 0,
      runic: None,
      dunes: None,
      total: 50 * COIN_VALUE,
    }
  );

  let no_sync_ord = TestServer::spawn_with_server_args(&core, &[], &["--no-sync"]);

  inscribe(&core, &dog);

  CommandBuilder::new("wallet balance")
    .dog(&no_sync_ord)
    .core(&core)
    .expected_exit_code(1)
    .expected_stderr("error: `dog server` 4 blocks behind `bitcoind`, consider using `--no-sync` to ignore this error\n")
    .run_and_extract_stdout();

  CommandBuilder::new("wallet --no-sync balance")
    .dog(&no_sync_ord)
    .core(&core)
    .expected_exit_code(1)
    .stderr_regex(r"error: output in wallet but not in dog server: [[:xdigit:]]{64}:\d+.*")
    .run_and_extract_stdout();
}

#[test]
fn runic_utxos_are_displayed_with_decimal_amount() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let dog = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-dunes"], &[]);

  create_wallet(&core, &dog);

  pretty_assert_eq!(
    CommandBuilder::new("--regtest --index-dunes wallet balance")
      .core(&core)
      .dog(&dog)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 0,
      ordinal: 0,
      runic: Some(0),
      dunes: Some(BTreeMap::new()),
      total: 0,
    }
  );

  let dune = Dune(RUNE);

  batch(
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

  pretty_assert_eq!(
    CommandBuilder::new("--regtest --index-dunes wallet balance")
      .core(&core)
      .dog(&dog)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 50 * COIN_VALUE * 7 - 20_000,
      ordinal: 10000,
      runic: Some(10_000),
      dunes: Some(
        vec![(
          SpacedDune { dune, spacers: 1 },
          Decimal {
            value: 1111,
            scale: 3,
          }
        )]
        .into_iter()
        .collect()
      ),
      total: 50 * COIN_VALUE * 7,
    }
  );
}
