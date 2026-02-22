use {super::*, dog::subcommand::wallet::balance::Output};

#[test]
fn authentication() {
  let core = mockcore::spawn();

  let dog = TestServer::spawn_with_server_args(
    &core,
    &["--server-username", "foo", "--server-password", "bar"],
    &[],
  );

  create_wallet(&core, &dog);

  assert_eq!(
    CommandBuilder::new("--server-username foo --server-password bar wallet balance")
      .core(&core)
      .dog(&dog)
      .run_and_deserialize_output::<Output>()
      .cardinal,
    0
  );

  core.mine_blocks(1);

  assert_eq!(
    CommandBuilder::new("--server-username foo --server-password bar wallet balance")
      .core(&core)
      .dog(&dog)
      .run_and_deserialize_output::<Output>(),
    Output {
      cardinal: 50 * COIN_VALUE,
      ordinal: 0,
      runic: None,
      dunes: None,
      total: 50 * COIN_VALUE,
    }
  );
}
