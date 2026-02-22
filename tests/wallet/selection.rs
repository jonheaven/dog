use super::*;

#[test]
fn inscribe_does_not_select_runic_utxos() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let dog = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-dunes"], &[]);

  create_wallet(&core, &dog);

  etch(&core, &dog, Dune(RUNE));

  drain(&core, &dog);

  CommandBuilder::new("--regtest --index-dunes wallet inscribe --fee-rate 0 --file foo.txt")
    .write("foo.txt", "FOO")
    .core(&core)
    .dog(&dog)
    .expected_exit_code(1)
    .expected_stderr("error: wallet contains no cardinal utxos\n")
    .run_and_extract_stdout();
}

#[test]
fn send_amount_does_not_select_runic_utxos() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let dog = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-dunes"], &[]);

  create_wallet(&core, &dog);

  etch(&core, &dog, Dune(RUNE));

  drain(&core, &dog);

  CommandBuilder::new("--regtest --index-dunes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 600sat")
    .core(&core)
    .dog(&dog)
    .expected_exit_code(1)
    .expected_stderr("error: not enough cardinal utxos\n")
    .run_and_extract_stdout();
}

#[test]
fn send_satpoint_does_not_send_runic_utxos() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let dog = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-dunes"], &[]);

  create_wallet(&core, &dog);

  core.mine_blocks_with_subsidy(1, 10000);

  let etched = etch(&core, &dog, Dune(RUNE));

  CommandBuilder::new(format!(
    "
        --regtest
        --index-dunes
        wallet
        send
        --fee-rate 1
        bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw
        {}:0
      ",
    etched.output.dune.unwrap().location.unwrap()
  ))
  .core(&core)
  .dog(&dog)
  .expected_stderr("error: runic outpoints may not be sent by satpoint\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn send_inscription_does_not_select_runic_utxos() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let dog = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-dunes"], &[]);

  create_wallet(&core, &dog);

  etch(&core, &dog, Dune(RUNE));

  let (id, _) = inscribe(&core, &dog);

  drain(&core, &dog);

  CommandBuilder::new(
    format!(
      "
        --regtest
        --index-dunes
        wallet
        send
        --postage 10000sat
        --fee-rate 1
        bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw
        {id}
      "))
    .core(&core)
    .dog(&dog)
    .expected_stderr("error: wallet does not contain enough cardinal UTXOs, please add additional funds to wallet.\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn mint_does_not_select_inscription() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let dog = TestServer::spawn_with_server_args(&core, &["--index-dunes", "--regtest"], &[]);

  create_wallet(&core, &dog);

  batch(
    &core,
    &dog,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 1,
        dune: SpacedDune {
          dune: Dune(RUNE),
          spacers: 0,
        },
        premine: "1000".parse().unwrap(),
        supply: "2000".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          cap: 1,
          amount: "1000".parse().unwrap(),
          offset: None,
          height: None,
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

  drain(&core, &dog);

  CommandBuilder::new(format!(
    "--chain regtest --index-dunes wallet mint --fee-rate 0 --dune {}",
    Dune(RUNE)
  ))
  .core(&core)
  .dog(&dog)
  .expected_exit_code(1)
  .expected_stderr("error: not enough cardinal utxos\n")
  .run_and_extract_stdout();
}

#[test]
fn sending_rune_does_not_send_inscription() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let dog = TestServer::spawn_with_server_args(&core, &["--index-dunes", "--regtest"], &[]);

  create_wallet(&core, &dog);

  core.mine_blocks_with_subsidy(1, 10000);

  let dune = Dune(RUNE);

  CommandBuilder::new("--chain regtest --index-dunes wallet inscribe --fee-rate 0 --file foo.txt")
    .write("foo.txt", "FOO")
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<Batch>();

  core.mine_blocks_with_subsidy(1, 10000);

  pretty_assert_eq!(
    CommandBuilder::new("--regtest --index-dunes wallet balance")
      .core(&core)
      .dog(&dog)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 10000,
      ordinal: 10000,
      runic: Some(0),
      dunes: Some(BTreeMap::new()),
      total: 20000,
    }
  );

  etch(&core, &dog, dune);

  drain(&core, &dog);

  CommandBuilder::new(format!(
    "
       --chain regtest
       --index-dunes
       wallet send
       --postage 11111sat
       --fee-rate 0
       bcrt1pyrmadgg78e38ewfv0an8c6eppk2fttv5vnuvz04yza60qau5va0saknu8k
       1000:{dune}
     ",
  ))
  .core(&core)
  .dog(&dog)
  .expected_exit_code(1)
  .expected_stderr("error: not enough cardinal utxos\n")
  .run_and_extract_stdout();
}

#[test]
fn split_does_not_select_inscribed_or_runic_utxos() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let dog = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-dunes"], &[]);

  create_wallet(&core, &dog);

  let dune = Dune(RUNE);

  etch(&core, &dog, dune);

  etch(&core, &dog, Dune(RUNE + 1));

  drain(&core, &dog);

  pretty_assert_eq!(
    CommandBuilder::new("--regtest wallet balance")
      .core(&core)
      .dog(&dog)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 0,
      ordinal: 20000,
      runic: Some(20000),
      dunes: Some(
        [
          (SpacedDune { dune, spacers: 0 }, "1000".parse().unwrap()),
          (
            SpacedDune {
              dune: Dune(RUNE + 1),
              spacers: 0
            },
            "1000".parse().unwrap()
          ),
        ]
        .into()
      ),
      total: 40000,
    }
  );

  CommandBuilder::new("--regtest wallet split --fee-rate 0 --splits splits.yaml")
    .core(&core)
    .dog(&dog)
    .write(
      "splits.yaml",
      format!(
        "
outputs:
- address: bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw
  value: 20000 sat
  dunes:
    {dune}: 1000
"
      ),
    )
    .expected_exit_code(1)
    .expected_stderr("error: not enough cardinal utxos\n")
    .run_and_extract_stdout();
}

#[test]
fn offer_create_does_not_select_non_cardinal_utxos() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let dog = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-dunes"], &[]);

  create_wallet(&core, &dog);

  let etch = etch(&core, &dog, Dune(RUNE));

  let inscription = etch.output.inscriptions[0].id;

  CommandBuilder::new(format!(
    "--regtest \
    --index-dunes \
    wallet \
    send \
    --fee-rate 0 \
    bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw \
    {inscription}"
  ))
  .core(&core)
  .dog(&dog)
  .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  drain(&core, &dog);

  CommandBuilder::new(format!(
    "--regtest --index-dunes wallet offer create --fee-rate 0 --inscription {inscription} --amount 1sat",
  ))
  .core(&core)
  .dog(&dog)
  .expected_exit_code(1)
  .expected_stderr("error: not enough cardinal utxos\n")
  .run_and_extract_stdout();
}
