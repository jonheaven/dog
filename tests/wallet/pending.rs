use {
  super::*,
  nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
  },
};

#[test]
fn wallet_pending() {
  let core = mockcore::builder().network(Network::Regtest).build();
  let dog = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-dunes"], &[]);

  create_wallet(&core, &dog);

  core.mine_blocks(1);

  let batchfile = batch::File {
    etching: Some(batch::Etching {
      divisibility: 0,
      dune: SpacedDune {
        dune: Dune(RUNE),
        spacers: 0,
      },
      supply: "1000".parse().unwrap(),
      premine: "1000".parse().unwrap(),
      symbol: '¢',
      ..default()
    }),
    inscriptions: vec![batch::Entry {
      file: Some("inscription.jpeg".into()),
      ..default()
    }],
    ..default()
  };

  let tempdir = Arc::new(TempDir::new().unwrap());

  {
    let mut spawn =
      CommandBuilder::new("--regtest --index-dunes wallet batch --fee-rate 0 --batch batch.yaml")
        .temp_dir(tempdir.clone())
        .write("batch.yaml", serde_yaml::to_string(&batchfile).unwrap())
        .write("inscription.jpeg", "inscription")
        .core(&core)
        .dog(&dog)
        .expected_exit_code(1)
        .spawn();

    let mut buffer = String::new();

    BufReader::new(spawn.child.stderr.as_mut().unwrap())
      .read_line(&mut buffer)
      .unwrap();

    assert_regex_match!(
      buffer,
      "Waiting for dune AAAAAAAAAAAAA commitment [[:xdigit:]]{64} to mature…\n"
    );

    core.mine_blocks(1);

    signal::kill(
      Pid::from_raw(spawn.child.id().try_into().unwrap()),
      Signal::SIGINT,
    )
    .unwrap();

    buffer.clear();

    BufReader::new(spawn.child.stderr.as_mut().unwrap())
      .read_line(&mut buffer)
      .unwrap();

    assert_eq!(
      buffer,
      "Shutting down gracefully. Press <CTRL-C> again to shutdown immediately.\n"
    );

    spawn.child.wait().unwrap();
  }

  let output = CommandBuilder::new("--regtest --index-dunes wallet pending")
    .temp_dir(tempdir)
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<Vec<dog::subcommand::wallet::pending::PendingOutput>>();

  assert_eq!(output.first().unwrap().dune.dune, Dune(RUNE));
}
