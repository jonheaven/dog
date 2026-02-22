use {
  super::*,
  dog::subcommand::wallet::{addresses::Output as AddressesOutput, sign::Output as SignOutput},
};

#[test]
fn sign() {
  let core = mockcore::spawn();

  let dog = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &dog);

  core.mine_blocks(1);

  let addresses = CommandBuilder::new("wallet addresses")
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<BTreeMap<Address<NetworkUnchecked>, Vec<AddressesOutput>>>();

  let address = addresses.first_key_value().unwrap().0;

  let text = "HelloWorld";

  let sign = CommandBuilder::new(format!(
    "wallet sign --signer {} --text {text}",
    address.clone().assume_checked(),
  ))
  .core(&core)
  .dog(&dog)
  .run_and_deserialize_output::<SignOutput>();

  assert_eq!(address, &sign.address);

  CommandBuilder::new(format!(
    "verify --address {} --text {text} --witness {}",
    address.clone().assume_checked(),
    sign.witness,
  ))
  .core(&core)
  .dog(&dog)
  .run_and_extract_stdout();
}

#[test]
fn sign_file() {
  let core = mockcore::spawn();

  let dog = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &dog);

  core.mine_blocks(1);

  let addresses = CommandBuilder::new("wallet addresses")
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<BTreeMap<Address<NetworkUnchecked>, Vec<AddressesOutput>>>();

  let address = addresses.first_key_value().unwrap().0;

  let sign = CommandBuilder::new(format!(
    "wallet sign --signer {} --file hello.txt",
    address.clone().assume_checked(),
  ))
  .write("hello.txt", "Hello World")
  .core(&core)
  .dog(&dog)
  .run_and_deserialize_output::<SignOutput>();

  assert_eq!(address, &sign.address);

  CommandBuilder::new(format!(
    "verify --address {} --file hello.txt --witness {}",
    address.clone().assume_checked(),
    sign.witness,
  ))
  .write("hello.txt", "Hello World")
  .core(&core)
  .dog(&dog)
  .run_and_extract_stdout();

  CommandBuilder::new(format!(
    "verify --address {} --file hello.txt --witness {}",
    address.clone().assume_checked(),
    sign.witness,
  ))
  .write("hello.txt", "FAIL")
  .core(&core)
  .dog(&dog)
  .expected_exit_code(1)
  .stderr_regex("error: Invalid signature.*")
  .run_and_extract_stdout();
}

#[test]
fn sign_for_inscription() {
  let core = mockcore::spawn();

  let dog = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &dog);

  let (inscription, _reveal) = inscribe(&core, &dog);

  core.mine_blocks(1);

  let addresses = CommandBuilder::new("wallet addresses")
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<BTreeMap<Address<NetworkUnchecked>, Vec<AddressesOutput>>>();

  let text = "HelloWorld";

  let sign = CommandBuilder::new(format!("wallet sign --signer {inscription} --text {text}",))
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<SignOutput>();

  assert!(addresses.contains_key(&sign.address));
}

#[test]
fn sign_for_output() {
  let core = mockcore::spawn();

  let dog = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &dog);

  core.mine_blocks(1);

  let addresses = CommandBuilder::new("wallet addresses")
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<BTreeMap<Address<NetworkUnchecked>, Vec<AddressesOutput>>>();

  let output = addresses.first_key_value().unwrap().1[0].output;

  let text = "HelloWorld";

  let sign = CommandBuilder::new(format!("wallet sign --signer {output} --text {text}",))
    .core(&core)
    .dog(&dog)
    .run_and_deserialize_output::<SignOutput>();

  assert!(addresses.contains_key(&sign.address));
}
