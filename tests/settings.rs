use super::*;

#[test]
fn default() {
  CommandBuilder::new("settings")
    .integration_test(false)
    .stdout_regex(
      r#"\{
  "dogecoin_data_dir": ".*(Dogecoin|dogecoin)",
  "dogecoin_rpc_limit": 12,
  "dogecoin_rpc_password": null,
  "dogecoin_rpc_url": "127.0.0.1:22555",
  "dogecoin_rpc_username": null,
  "chain": "dogecoin",
  "commit_interval": 5000,
  "config": null,
  "config_dir": null,
  "cookie_file": ".*\.cookie",
  "data_dir": ".*",
  "height_limit": null,
  "hidden": \[\],
  "http_port": null,
  "index": ".*index\.redb",
  "index_addresses": false,
  "index_cache_size": \d+,
  "index_dunes": false,
  "index_koinu": false,
  "index_transactions": false,
  "only_protocols": null,
  "integration_test": false,
  "max_savepoints": 2,
  "no_index_inscriptions": false,
  "savepoint_interval": 10,
  "server_password": null,
  "server_url": null,
  "server_username": null
\}
"#,
    )
    .run_and_extract_stdout();
}

#[test]
fn config_is_loaded_from_config_option() {
  let tempdir = TempDir::new().unwrap();

  let config = tempdir.path().join("dog.yaml");

  fs::write(&config, "chain: dogecoin-regtest").unwrap();

  CommandBuilder::new(format!("--config {} settings", config.to_str().unwrap()))
    .stdout_regex(
      r#".*
  "chain": "dogecoin-regtest",
.*"#,
    )
    .run_and_extract_stdout();
}

#[test]
fn config_invalid_error_message() {
  let tempdir = TempDir::new().unwrap();

  let config = tempdir.path().join("dog.yaml");

  fs::write(&config, "foo").unwrap();

  CommandBuilder::new(format!("--config {} settings", config.to_str().unwrap()))
    .stderr_regex("error: failed to deserialize config file `.*dog.yaml`\n\nbecause:.*")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn config_not_found_error_message() {
  let tempdir = TempDir::new().unwrap();

  let config = tempdir.path().join("dog.yaml");

  CommandBuilder::new(format!("--config {} settings", config.to_str().unwrap()))
    .stderr_regex("error: failed to open config file `.*dog.yaml`\n\nbecause:.*")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn config_is_loaded_from_config_dir() {
  let tempdir = TempDir::new().unwrap();

  fs::write(tempdir.path().join("dog.yaml"), "chain: dogecoin-regtest").unwrap();

  CommandBuilder::new(format!(
    "--config-dir {} settings",
    tempdir.path().to_str().unwrap()
  ))
  .stdout_regex(
    r#".*
  "chain": "dogecoin-regtest",
.*"#,
  )
  .run_and_extract_stdout();
}

#[test]
fn config_is_loaded_from_data_dir() {
  CommandBuilder::new("settings")
    .write("dog.yaml", "chain: dogecoin-regtest")
    .stdout_regex(
      r#".*
  "chain": "dogecoin-regtest",
.*"#,
    )
    .run_and_extract_stdout();
}

#[test]
fn env_is_loaded() {
  CommandBuilder::new("settings")
    .stdout_regex(
      r#".*
  "chain": "dogecoin",
.*"#,
    )
    .run_and_extract_stdout();

  CommandBuilder::new("settings")
    .env("DOG_CHAIN", "dogecoin-regtest")
    .stdout_regex(
      r#".*
  "chain": "dogecoin-regtest",
.*"#,
    )
    .run_and_extract_stdout();
}

#[cfg(unix)]
#[test]
fn invalid_env_error_message() {
  use std::os::unix::ffi::OsStringExt;

  CommandBuilder::new("settings")
    .env("DOG_BAR", OsString::from_vec(b"\xFF".into()))
    .stderr_regex("error: environment variable `DOG_BAR` not valid unicode: `�`\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}
