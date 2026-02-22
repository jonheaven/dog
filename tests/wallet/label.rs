use super::*;

#[test]
fn label() {
  let core = mockcore::spawn();

  let dog = TestServer::spawn_with_server_args(&core, &["--index-koinu"], &[]);

  create_wallet(&core, &dog);

  core.mine_blocks(2);

  let (inscription, _reveal) = inscribe(&core, &dog);

  let output = CommandBuilder::new("wallet label")
    .core(&core)
    .dog(&dog)
    .stdout_regex(".*")
    .run_and_extract_stdout();

  assert!(
    output.contains(r#"\"name\":\"nvtcsezkbth\",\"number\":5000000000,\"rarity\":\"uncommon\""#)
  );

  assert!(
    output.contains(r#"\"name\":\"nvtccadxgaz\",\"number\":10000000000,\"rarity\":\"uncommon\""#)
  );

  assert!(output.contains(&inscription.to_string()));
}
