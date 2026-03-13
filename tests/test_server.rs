use {
  super::*,
  axum_server::Handle,
  bitcoincore_rpc::{Auth, Client, RpcApi},
  dog::{Index, parse_dog_server_args},
  reqwest::blocking::Response,
  std::net::SocketAddr,
  sysinfo::System,
};

pub(crate) struct TestServer {
  client: Client,
  dog_server_handle: Handle<SocketAddr>,
  port: u16,
  #[allow(unused)]
  tempdir: TempDir,
}

impl TestServer {
  pub(crate) fn spawn(core: &mockcore::Handle) -> Self {
    Self::spawn_with_server_args(core, &[], &[])
  }

  pub(crate) fn spawn_with_args(core: &mockcore::Handle, dog_args: &[&str]) -> Self {
    Self::spawn_with_server_args(core, dog_args, &[])
  }

  pub(crate) fn spawn_with_server_args(
    core: &mockcore::Handle,
    dog_args: &[&str],
    dog_server_args: &[&str],
  ) -> Self {
    let tempdir = TempDir::new().unwrap();

    let cookiefile = tempdir.path().join("cookie");

    fs::write(&cookiefile, "username:password").unwrap();

    let (settings, server) = parse_dog_server_args(&format!(
      "dog --dogecoin-rpc-url {} --cookie-file {} --dogecoin-data-dir {} --datadir {} {} server {} --http-port 0 --address 127.0.0.1",
      core.url(),
      cookiefile.to_str().unwrap(),
      tempdir.path().display(),
      tempdir.path().display(),
      dog_args.join(" "),
      dog_server_args.join(" "),
    ));

    let index = Arc::new(Index::open(&settings).unwrap());
    let dog_server_handle = Handle::new();

    let (tx, rx) = std::sync::mpsc::channel();

    {
      let index = index.clone();
      let dog_server_handle = dog_server_handle.clone();
      thread::spawn(|| {
        server
          .run(settings, index, dog_server_handle, Some(tx))
          .unwrap()
      });
    }

    let port = rx.recv().unwrap();

    let client = Client::new(&core.url(), Auth::None).unwrap();

    Self {
      client,
      dog_server_handle,
      port,
      tempdir,
    }
  }

  pub(crate) fn url(&self) -> Url {
    format!("http://127.0.0.1:{}", self.port).parse().unwrap()
  }

  #[track_caller]
  pub(crate) fn assert_response_regex(&self, path: impl AsRef<str>, regex: impl AsRef<str>) {
    self.sync_server();
    let path = path.as_ref();
    let response = reqwest::blocking::get(self.url().join(path.as_ref()).unwrap()).unwrap();
    let status = response.status();
    assert_eq!(status, StatusCode::OK, "bad status for {path}: {status}");
    let text = response.text().unwrap();
    assert_regex_match!(text, regex.as_ref());
  }

  #[track_caller]
  pub(crate) fn assert_response(&self, path: impl AsRef<str>, expected_response: &str) {
    self.sync_server();
    let response = reqwest::blocking::get(self.url().join(path.as_ref()).unwrap()).unwrap();
    assert_eq!(
      response.status(),
      StatusCode::OK,
      "{}",
      response.text().unwrap()
    );
    pretty_assert_eq!(response.text().unwrap(), expected_response);
  }

  #[track_caller]
  pub(crate) fn assert_html(
    &self,
    path: impl AsRef<str>,
    chain: Chain,
    content: impl dog::templates::PageContent,
  ) {
    self.sync_server();
    let response = reqwest::blocking::get(self.url().join(path.as_ref()).unwrap()).unwrap();

    assert_eq!(
      response.status(),
      StatusCode::OK,
      "{}",
      response.text().unwrap()
    );

    let expected_response = dog::templates::PageHtml::new(
      content,
      Arc::new(dog::subcommand::server::ServerConfig {
        chain,
        domain: Some(System::host_name().unwrap()),
        ..Default::default()
      }),
    )
    .to_string();

    pretty_assert_eq!(response.text().unwrap(), expected_response);
  }

  pub(crate) fn request(&self, path: impl AsRef<str>) -> Response {
    self.sync_server();

    reqwest::blocking::get(self.url().join(path.as_ref()).unwrap()).unwrap()
  }

  pub(crate) fn json_request(&self, path: impl AsRef<str>) -> Response {
    self.sync_server();

    let client = reqwest::blocking::Client::new();

    client
      .get(self.url().join(path.as_ref()).unwrap())
      .header(reqwest::header::ACCEPT, "application/json")
      .send()
      .unwrap()
  }

  pub(crate) fn sync_server(&self) {
    let chain_block_count = self.client.get_block_count().unwrap() + 1;
    let response = reqwest::blocking::get(self.url().join("/update").unwrap()).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().unwrap().parse::<u64>().unwrap() >= chain_block_count);
  }
}

impl Drop for TestServer {
  fn drop(&mut self) {
    self.dog_server_handle.shutdown();
  }
}
