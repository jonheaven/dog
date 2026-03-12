function formatNumber(value) {
  return new Intl.NumberFormat().format(value ?? 0);
}

function formatRate(value) {
  return `${Number(value ?? 0).toFixed(2)} inscriptions/sec`;
}

function formatMemory(bytes) {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let nextValue = Number(bytes ?? 0);
  let unitIndex = 0;

  while (nextValue >= 1024 && unitIndex < units.length - 1) {
    nextValue /= 1024;
    unitIndex += 1;
  }

  if (unitIndex === 0) {
    return `${Math.round(nextValue)} ${units[unitIndex]}`;
  }

  return `${nextValue.toFixed(1)} ${units[unitIndex]}`;
}

function formatDuration(totalSeconds) {
  const seconds = Math.max(0, Math.floor(Number(totalSeconds ?? 0)));
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const remainingSeconds = seconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }

  if (minutes > 0) {
    return `${minutes}m ${remainingSeconds}s`;
  }

  return `${remainingSeconds}s`;
}

function createFeedItem(item) {
  const listItem = document.createElement("li");
  listItem.className = "monitor-feed-item";
  listItem.dataset.feedKey = `${item.kind}:${item.title}:${item.timestamp}`;

  const meta = document.createElement("div");
  meta.className = "monitor-feed-meta";

  const kind = document.createElement("span");
  kind.className = `monitor-feed-kind monitor-feed-kind-${item.kind}`;
  kind.textContent = item.kind;
  meta.appendChild(kind);

  const height = document.createElement("span");
  height.className = "monitor-feed-height";
  height.textContent = item.height ? `height ${item.height}` : "watching live";
  meta.appendChild(height);

  const link = document.createElement("a");
  link.className = "monitor-feed-link";
  link.href = item.link;
  link.textContent = item.title;

  const copy = document.createElement("p");
  copy.className = "monitor-feed-copy";
  copy.textContent = item.subtitle;

  listItem.append(meta, link, copy);
  return listItem;
}

function updateHeightNode(node, height) {
  if (!node) {
    return;
  }

  node.replaceChildren();

  if (typeof height === "number") {
    const link = document.createElement("a");
    link.href = `/block/${height}`;
    link.textContent = formatNumber(height);
    node.appendChild(link);
    return;
  }

  node.textContent = "Waiting for blocks";
}

async function fetchJson(path) {
  const response = await fetch(path, { cache: "no-store" });

  if (!response.ok) {
    throw new Error(`Request failed for ${path}: ${response.status}`);
  }

  return response.json();
}

function initLiveStatusBar() {
  const header = document.querySelector("body > header");

  if (!header) {
    return;
  }

  const bar = document.createElement("div");
  bar.className = "live-status-bar";
  bar.innerHTML = `
    <a class="live-status-bar__inner" href="/monitor">
      <span class="live-status-bar__pill">Indexing live</span>
      <span class="live-status-bar__metric" data-status-bar-height>Height ...</span>
      <span class="live-status-bar__metric" data-status-bar-rate>0.00 inscriptions/sec</span>
      <span class="live-status-bar__metric" data-status-bar-lag>lag ...</span>
      <span class="live-status-bar__action">Open monitor</span>
    </a>
  `;
  document.body.insertBefore(bar, header);

  const heightNode = bar.querySelector("[data-status-bar-height]");
  const rateNode = bar.querySelector("[data-status-bar-rate]");
  const lagNode = bar.querySelector("[data-status-bar-lag]");
  let lastSample = null;

  async function refresh() {
    try {
      const status = await fetchJson("/api/status");
      const now = Date.now();

      let inscriptionsPerSecond = status.inscriptions_per_second;

      if (
        lastSample &&
        typeof status.inscriptions === "number" &&
        typeof lastSample.inscriptions === "number"
      ) {
        const elapsedSeconds = Math.max((now - lastSample.capturedAt) / 1000, 1);
        inscriptionsPerSecond = (status.inscriptions - lastSample.inscriptions) / elapsedSeconds;
      }

      heightNode.textContent =
        typeof status.height === "number"
          ? `Height ${formatNumber(status.height)}`
          : "Height pending";
      rateNode.textContent = formatRate(Math.max(inscriptionsPerSecond, 0));
      lagNode.textContent =
        status.lag_blocks > 0 ? `lag ${formatNumber(status.lag_blocks)}` : "synced";

      lastSample = {
        capturedAt: now,
        inscriptions: status.inscriptions,
      };
    } catch (error) {
      console.error(error);
      lagNode.textContent = "monitor unavailable";
    }
  }

  refresh();
  window.setInterval(refresh, 4000);
}

function initMonitorPage() {
  const root = document.getElementById("monitor-root");

  if (!root) {
    return;
  }

  const feedList = document.getElementById("monitor-feed-list");
  let seenFeedKeys = new Set(
    Array.from(feedList?.querySelectorAll("[data-feed-key]") ?? []).map((node) => node.dataset.feedKey)
  );

  function setText(selector, value) {
    const node = root.querySelector(selector);

    if (node) {
      node.textContent = value;
    }
  }

  function applySnapshot(snapshot) {
    updateHeightNode(root.querySelector("[data-live-height]"), snapshot.status.height);
    setText("[data-live-status]", snapshot.status.syncing ? "Syncing live" : "Fully synced");
    setText("[data-live-chain-tip]", formatNumber(snapshot.status.chain_tip));
    setText("[data-live-lag]", formatNumber(snapshot.status.lag_blocks));
    setText("[data-live-block-rate]", Number(snapshot.status.blocks_per_second).toFixed(2));
    setText(
      "[data-live-inscription-rate]",
      Number(snapshot.status.inscriptions_per_second).toFixed(2)
    );
    setText("[data-live-active-protocols]", snapshot.status.active_protocols.join(", "));
    setText("[data-live-total-indexed]", formatNumber(snapshot.stats.total_indexed));
    setText("[data-live-memory]", formatMemory(snapshot.stats.memory_usage_bytes));
    setText("[data-live-reorgs]", formatNumber(snapshot.stats.reorg_count));
    setText("[data-live-webhooks]", formatNumber(snapshot.stats.webhook_deliveries));
    setText("[data-live-inscriptions]", formatNumber(snapshot.status.inscriptions));
    setText("[data-live-blessed]", formatNumber(snapshot.stats.blessed_inscriptions));
    setText("[data-live-cursed]", formatNumber(snapshot.stats.cursed_inscriptions));
    setText("[data-live-dunes]", formatNumber(snapshot.status.dunes));
    setText("[data-live-dogemaps]", formatNumber(snapshot.status.dogemaps));
    setText("[data-live-dogespells]", formatNumber(snapshot.status.dogespells));
    setText("[data-live-dmp]", formatNumber(snapshot.status.dmp));
    setText("[data-live-dogelotto]", formatNumber(snapshot.status.dogelotto));
    setText("[data-live-uptime]", formatDuration(snapshot.stats.uptime_seconds));
    setText("[data-live-initial-sync]", formatDuration(snapshot.stats.initial_sync_seconds));

    if (!feedList) {
      return;
    }

    const emptyState = feedList.querySelector("[data-feed-key='empty']");
    if (snapshot.feed.length > 0 && emptyState) {
      emptyState.remove();
      seenFeedKeys.delete("empty");
    }

    for (const item of snapshot.feed) {
      const key = `${item.kind}:${item.title}:${item.timestamp}`;

      if (seenFeedKeys.has(key)) {
        continue;
      }

      feedList.prepend(createFeedItem(item));
      seenFeedKeys.add(key);
    }

    while (feedList.children.length > 30) {
      const lastChild = feedList.lastElementChild;
      if (!lastChild) {
        break;
      }

      seenFeedKeys.delete(lastChild.dataset.feedKey);
      lastChild.remove();
    }
  }

  async function refresh() {
    try {
      applySnapshot(await fetchJson("/api/monitor"));
    } catch (error) {
      console.error(error);
    }
  }

  refresh();
  window.setInterval(refresh, 5000);
}

addEventListener("DOMContentLoaded", () => {
  for (const time of document.body.getElementsByTagName("time")) {
    time.setAttribute("title", new Date(time.textContent));
  }

  const next = document.querySelector("a.next");
  const prev = document.querySelector("a.prev");

  window.addEventListener("keydown", (event) => {
    if (document.activeElement?.tagName === "INPUT") {
      return;
    }

    switch (event.key) {
      case "ArrowRight":
        if (next) {
          window.location = next.href;
        }
        break;
      case "ArrowLeft":
        if (prev) {
          window.location = prev.href;
        }
        break;
      default:
        break;
    }
  });

  const search = document.querySelector('form[action="/search"]');
  const query = search?.querySelector('input[name="query"]');

  search?.addEventListener("submit", (event) => {
    if (!query?.value) {
      event.preventDefault();
    }
  });

  const collapse = document.getElementsByClassName("collapse");
  const context = document.createElement("canvas").getContext("2d");

  function resize() {
    for (const node of collapse) {
      if (!("original" in node.dataset)) {
        node.dataset.original = node.textContent.trim();
      }

      const original = node.dataset.original;
      const width = node.clientWidth || node.parentNode.getBoundingClientRect().width;
      const length = original.length;

      context.font = window.getComputedStyle(node).font;
      const capacity = width / (context.measureText(original).width / length);

      if (capacity >= length) {
        node.textContent = original;
        continue;
      }

      const count = Math.floor((capacity - 3) / 2);
      const start = original.substring(0, count);
      const end = original.substring(length - count);
      node.textContent = `${start}...${end}`;
    }
  }

  function copy(event) {
    if (
      "original" in event.target.dataset &&
      window.getSelection().toString().includes("...")
    ) {
      event.clipboardData.setData("text/plain", event.target.dataset.original);
      event.preventDefault();
    }
  }

  addEventListener("resize", resize);
  addEventListener("copy", copy);

  resize();
  initLiveStatusBar();
  initMonitorPage();

  if (document.getElementById("wallet-tools-root")) {
    import("/static/wallet.js").catch((error) => {
      console.error("Failed to load wallet tools", error);
    });
  }
});
