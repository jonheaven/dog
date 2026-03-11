use super::*;
use reqwest::blocking::Client;
use reqwest::header::ACCEPT;

#[derive(Debug, Parser)]
pub(crate) struct Parity {
  #[arg(
    long,
    default_value = "http://127.0.0.1:8080",
    help = "Base URL for kabosu"
  )]
  kabosu_url: String,
  #[arg(
    long,
    default_value = "http://127.0.0.1",
    help = "Base URL for dog server"
  )]
  dog_url: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct EndpointCheck {
  pub name: String,
  pub url: String,
  pub ok: bool,
  pub status: u16,
  pub body: serde_json::Value,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub ok: bool,
  pub mismatches: Vec<String>,
  pub checks: Vec<EndpointCheck>,
}

impl Parity {
  pub(crate) fn run(self, _settings: Settings) -> SubcommandResult {
    let client = Client::builder().build()?;
    let kabosu_url = self.kabosu_url.trim_end_matches('/').to_string();
    let dog_url = self.dog_url.trim_end_matches('/').to_string();

    let mut checks = Vec::new();

    checks.push(fetch_json(
      &client,
      "kabosu-health",
      &format!("{kabosu_url}/v1/system/health"),
    )?);
    checks.push(fetch_json(
      &client,
      "kabosu-sync",
      &format!("{kabosu_url}/v1/system/sync"),
    )?);
    checks.push(fetch_json(
      &client,
      "dog-health",
      &format!("{dog_url}/health"),
    )?);
    checks.push(fetch_json(
      &client,
      "dog-status",
      &format!("{dog_url}/status"),
    )?);

    let mut mismatches = Vec::new();

    let kabosu_sync_height = checks
      .iter()
      .find(|check| check.name == "kabosu-sync")
      .and_then(|check| check.body.get("latestIndexedBlock"))
      .and_then(|value| value.as_u64());

    let dog_status_height = checks
      .iter()
      .find(|check| check.name == "dog-status")
      .and_then(|check| check.body.get("height"))
      .and_then(|value| value.as_u64());

    if let (Some(kabosu_height), Some(dog_height)) = (kabosu_sync_height, dog_status_height) {
      if kabosu_height != dog_height {
        mismatches.push(format!(
          "Indexed height drift detected: kabosu={kabosu_height}, dog={dog_height}"
        ));
      }
    } else {
      mismatches.push("Unable to compare index heights from kabosu and dog responses".to_string());
    }

    if checks.iter().any(|check| !check.ok) {
      mismatches.push("One or more parity endpoints returned a non-success response".to_string());
    }

    Ok(Some(Box::new(Output {
      ok: mismatches.is_empty(),
      mismatches,
      checks,
    })))
  }
}

fn fetch_json(client: &Client, name: &str, url: &str) -> Result<EndpointCheck> {
  let response = client.get(url).header(ACCEPT, "application/json").send()?;

  let status = response.status();
  let text = response.text()?;
  let body = serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!({ "raw": text }));

  Ok(EndpointCheck {
    name: name.to_string(),
    url: url.to_string(),
    ok: status.is_success(),
    status: status.as_u16(),
    body,
  })
}
