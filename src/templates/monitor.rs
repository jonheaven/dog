use super::*;

#[derive(Boilerplate)]
pub(crate) struct MonitorHtml {
  pub(crate) monitor: crate::api::MonitorJson,
}

impl MonitorHtml {
  fn height_label(&self) -> String {
    self
      .monitor
      .status
      .height
      .map(|height| height.to_string())
      .unwrap_or_else(|| "Waiting for blocks".into())
  }

  fn blocks_per_second(&self) -> String {
    format!("{:.2}", self.monitor.status.blocks_per_second)
  }

  fn inscriptions_per_second(&self) -> String {
    format!("{:.2}", self.monitor.status.inscriptions_per_second)
  }

  fn memory_usage(&self) -> String {
    format_bytes(self.monitor.stats.memory_usage_bytes)
  }

  fn sync_label(&self) -> &'static str {
    if self.monitor.status.syncing {
      "Syncing live"
    } else {
      "Fully synced"
    }
  }
}

fn format_bytes(bytes: u64) -> String {
  const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

  if bytes < 1024 {
    return format!("{bytes} B");
  }

  let mut value = bytes as f64;
  let mut unit = 0usize;

  while value >= 1024.0 && unit < UNITS.len() - 1 {
    value /= 1024.0;
    unit += 1;
  }

  format!("{value:.1} {}", UNITS[unit])
}

impl PageContent for MonitorHtml {
  fn title(&self) -> String {
    "Live Monitor".into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn html() {
    assert_regex_match!(
      MonitorHtml {
        monitor: crate::api::MonitorJson {
          status: crate::api::LiveStatusJson {
            chain: Chain::Dogecoin,
            height: Some(5_000_001),
            chain_tip: 5_000_100,
            lag_blocks: 99,
            status: "syncing".into(),
            syncing: true,
            blocks_per_second: 1.5,
            inscriptions_per_second: 3.25,
            inscriptions: 42,
            dunes: 2,
            dogemaps: 3,
            dogespells: 0,
            dmp: 0,
            dogelotto: 0,
            active_protocols: vec!["dns".into(), "drc20".into(), "dogemap".into()],
            updated_at: 0,
          },
          stats: crate::api::MonitorStatsJson {
            total_indexed: 47,
            blessed_inscriptions: 40,
            cursed_inscriptions: 2,
            memory_usage_bytes: 512 * 1024 * 1024,
            reorg_count: 1,
            webhook_deliveries: 0,
            initial_sync_seconds: 90,
            uptime_seconds: 120,
          },
          feed: vec![crate::api::MonitorFeedItem {
            kind: "inscription".into(),
            title: "Inscription 42".into(),
            subtitle: "height 5000001".into(),
            link: "/inscription/0".into(),
            height: Some(5_000_001),
            timestamp: 0,
          }],
        },
      }
      .to_string()
      .unindent(),
      ".*Live Index Monitor.*Dogestash console.*Immediate additive feed.*Inscription 42.*"
    );
  }
}
