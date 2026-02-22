use {super::*, dog::subcommand::wallet::receive};

#[test]
fn receive() {
  let core = mockcore::spawn();
  let dog = TestServer::spawn(&core);

  create_wallet(&core, &dog);

  let output = CommandBuilder::new("wallet receive")
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<receive::Output>();

  assert!(
    output
      .addresses
      .first()
      .unwrap()
      .is_valid_for_network(Network::Bitcoin)
  );
}
