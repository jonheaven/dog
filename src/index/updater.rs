use {
  self::{inscription_updater::InscriptionUpdater, dune_updater::DuneUpdater},
  super::{fetcher::Fetcher, *},
  futures::future::try_join_all,
  tokio::sync::{
    broadcast::{self, error::TryRecvError},
    mpsc::{self},
  },
};

mod blk_reader;
mod inscription_updater;
mod dune_updater;

pub(crate) struct BlockData {
  pub(crate) header: Header,
  pub(crate) txdata: Vec<(Transaction, Txid)>,
}

impl From<Block> for BlockData {
  fn from(block: Block) -> Self {
    BlockData {
      header: block.header,
      txdata: block
        .txdata
        .into_iter()
        .map(|transaction| {
          let txid = transaction.compute_txid();
          (transaction, txid)
        })
        .collect(),
    }
  }
}

pub(crate) struct Updater<'index> {
  pub(super) height: u32,
  pub(super) index: &'index Index,
  pub(super) outputs_cached: u64,
  pub(super) outputs_traversed: u64,
  pub(super) sat_ranges_since_flush: u64,
}

impl Updater<'_> {
  pub(crate) fn update_index(&mut self, mut wtx: WriteTransaction) -> Result {
    let start = Instant::now();
    let starting_height = u32::try_from(self.index.client.get_block_count()?).unwrap() + 1;
    let starting_index_height = self.height;

    wtx
      .open_table(WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP)?
      .insert(
        &self.height,
        &SystemTime::now()
          .duration_since(SystemTime::UNIX_EPOCH)
          .map(|duration| duration.as_millis())
          .unwrap_or(0),
      )?;

    let mut progress_bar = if cfg!(test)
      || log_enabled!(log::Level::Info)
      || starting_height <= self.height
      || self.index.settings.integration_test()
    {
      None
    } else {
      let progress_bar = ProgressBar::new(starting_height.into());
      progress_bar.set_position(self.height.into());
      progress_bar.set_style(
        ProgressStyle::with_template("[indexing blocks] {wide_bar} {pos}/{len}").unwrap(),
      );
      Some(progress_bar)
    };

    let rx = Self::fetch_blocks_from(self.index, self.height)?;

    let (mut output_sender, mut txout_receiver) = Self::spawn_fetcher(self.index)?;

    let mut uncommitted = 0;
    let mut utxo_cache = HashMap::new();
    while let Ok(block) = rx.recv() {
      self.index_block(
        &mut output_sender,
        &mut txout_receiver,
        &mut wtx,
        block,
        &mut utxo_cache,
      )?;

      if let Some(progress_bar) = &mut progress_bar {
        progress_bar.inc(1);

        if progress_bar.position() > progress_bar.length().unwrap() {
          if let Ok(count) = self.index.client.get_block_count() {
            progress_bar.set_length(count + 1);
          } else {
            log::warn!("Failed to fetch latest block height");
          }
        }
      }

      uncommitted += 1;

      if uncommitted == self.index.settings.commit_interval()
        || (!self.index.settings.integration_test()
          && Reorg::is_savepoint_required(self.index, self.height)?)
      {
        self.commit(wtx, utxo_cache)?;
        utxo_cache = HashMap::new();
        uncommitted = 0;
        wtx = self.index.begin_write()?;
        let height = wtx
          .open_table(HEIGHT_TO_BLOCK_HEADER)?
          .range(0..)?
          .next_back()
          .transpose()?
          .map(|(height, _hash)| height.value() + 1)
          .unwrap_or(0);
        if height != self.height {
          // another update has run between committing and beginning the new
          // write transaction
          break;
        }
        wtx
          .open_table(WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP)?
          .insert(
            &self.height,
            &SystemTime::now()
              .duration_since(SystemTime::UNIX_EPOCH)?
              .as_millis(),
          )?;
      }

      if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
        break;
      }
    }

    if starting_index_height == 0 && self.height > 0 {
      wtx.open_table(STATISTIC_TO_COUNT)?.insert(
        Statistic::InitialSyncTime.key(),
        &u64::try_from(start.elapsed().as_micros())?,
      )?;
    }

    if uncommitted > 0 {
      self.commit(wtx, utxo_cache)?;
    }

    if let Some(progress_bar) = &mut progress_bar {
      progress_bar.finish_and_clear();
    }

    Ok(())
  }

  fn fetch_blocks_from(
    index: &Index,
    mut height: u32,
  ) -> Result<std::sync::mpsc::Receiver<BlockData>> {
    let (tx, rx) = std::sync::mpsc::sync_channel(32);

    let first_index_height = index.first_index_height;

    let height_limit = index.height_limit;

    let client = index.settings.bitcoin_rpc_client(None)?;

    // Try to open a direct .blk file reader for fast initial sync.
    // Falls back to RPC automatically for tip blocks not yet flushed to disk.
    let fast_reader: Option<blk_reader::BlkReader> = index.settings
      .dogecoin_blocks_dir()
      .and_then(|dir| match blk_reader::BlkReader::open(&dir) {
        Ok(reader) => reader,
        Err(e) => {
          log::warn!("BlkReader: unexpected error: {e}");
          None
        }
      });

    thread::spawn(move || {
      loop {
        if let Some(height_limit) = height_limit
          && height >= height_limit
        {
          break;
        }

        // Fast path: read directly from .blk files on disk
        if let Some(ref reader) = fast_reader {
          match reader.get(height) {
            Ok(Some(block)) => {
              if let Err(err) = tx.send(block.into()) {
                log::info!("Block receiver disconnected: {err}");
                break;
              }
              height += 1;
              continue;
            }
            Ok(None) => {
              // Block not in .blk index yet (tip) — fall through to RPC
            }
            Err(e) => {
              log::warn!("BlkReader error at height {height}, retrying via RPC: {e}");
            }
          }
        }

        // RPC path: used for tip blocks and when BlkReader is unavailable
        match Self::get_block_with_retries(&client, height, first_index_height) {
          Ok(Some(block)) => {
            if let Err(err) = tx.send(block.into()) {
              log::info!("Block receiver disconnected: {err}");
              break;
            }
            height += 1;
          }
          Ok(None) => break,
          Err(err) => {
            log::error!("failed to fetch block {height}: {err}");
            break;
          }
        }
      }
    });

    Ok(rx)
  }

  fn get_block_with_retries(
    client: &Client,
    height: u32,
    first_index_height: u32,
  ) -> Result<Option<Block>> {
    let mut errors = 0;
    loop {
      match client
        .get_block_hash(height.into())
        .into_option()
        .and_then(|option| {
          option
            .map(|hash| {
              if height >= first_index_height {
                Ok(client.get_block(&hash)?)
              } else {
                Ok(Block {
                  header: client.get_block_header(&hash)?,
                  txdata: Vec::new(),
                })
              }
            })
            .transpose()
        }) {
        Err(err) => {
          if cfg!(test) {
            return Err(err);
          }

          errors += 1;
          let seconds = 1 << errors;
          log::warn!("failed to fetch block {height}, retrying in {seconds}s: {err}");

          if seconds > 120 {
            log::error!("would sleep for more than 120s, giving up");
            return Err(err);
          }

          thread::sleep(Duration::from_secs(seconds));
        }
        Ok(result) => return Ok(result),
      }
    }
  }

  fn spawn_fetcher(index: &Index) -> Result<(mpsc::Sender<OutPoint>, broadcast::Receiver<TxOut>)> {
    let fetcher = Fetcher::new(&index.settings)?;

    // A block probably has no more than 20k inputs
    const CHANNEL_BUFFER_SIZE: usize = 20_000;

    // Batch 2048 missing inputs at a time, arbitrarily chosen size
    const BATCH_SIZE: usize = 2048;

    let (outpoint_sender, mut outpoint_receiver) = mpsc::channel::<OutPoint>(CHANNEL_BUFFER_SIZE);

    let (txout_sender, txout_receiver) = broadcast::channel::<TxOut>(CHANNEL_BUFFER_SIZE);

    // Default rpcworkqueue in bitcoind is 16, meaning more than 16 concurrent requests will be rejected.
    // Since we are already requesting blocks on a separate thread, and we don't want to break if anything
    // else runs a request, we keep this to 12.
    let parallel_requests: usize = index.settings.bitcoin_rpc_limit().try_into().unwrap();

    thread::spawn(move || {
      let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
      rt.block_on(async move {
        loop {
          let Some(outpoint) = outpoint_receiver.recv().await else {
            log::debug!("Outpoint channel closed");
            return;
          };

          // There's no try_iter on tokio::sync::mpsc::Receiver like std::sync::mpsc::Receiver.
          // So we just loop until BATCH_SIZE doing try_recv until it returns None.
          let mut outpoints = vec![outpoint];
          for _ in 0..BATCH_SIZE - 1 {
            let Ok(outpoint) = outpoint_receiver.try_recv() else {
              break;
            };
            outpoints.push(outpoint);
          }

          // Break outputs into chunks for parallel requests
          let chunk_size = (outpoints.len() / parallel_requests) + 1;
          let mut futs = Vec::with_capacity(parallel_requests);
          for chunk in outpoints.chunks(chunk_size) {
            let txids = chunk.iter().map(|outpoint| outpoint.txid).collect();
            let fut = fetcher.get_transactions(txids);
            futs.push(fut);
          }

          let txs = match try_join_all(futs).await {
            Ok(txs) => txs,
            Err(e) => {
              log::error!("Couldn't receive txs {e}");
              return;
            }
          };

          // Send all tx outputs back in order
          for (i, tx) in txs.iter().flatten().enumerate() {
            let Ok(_) =
              txout_sender.send(tx.output[usize::try_from(outpoints[i].vout).unwrap()].clone())
            else {
              log::error!("Value channel closed unexpectedly");
              return;
            };
          }
        }
      })
    });

    Ok((outpoint_sender, txout_receiver))
  }

  fn index_block(
    &mut self,
    output_sender: &mut mpsc::Sender<OutPoint>,
    txout_receiver: &mut broadcast::Receiver<TxOut>,
    wtx: &mut WriteTransaction,
    block: BlockData,
    utxo_cache: &mut HashMap<OutPoint, UtxoEntryBuf>,
  ) -> Result<()> {
    Reorg::detect_reorg(&block, self.height, self.index)?;

    let start = Instant::now();
    let mut sat_ranges_written = 0;
    let mut outputs_in_block = 0;

    log::info!(
      "Block {} at {} with {} transactions…",
      self.height,
      timestamp(block.header.time.into()),
      block.txdata.len()
    );

    let mut height_to_block_header = wtx.open_table(HEIGHT_TO_BLOCK_HEADER)?;
    let mut inscription_id_to_sequence_number =
      wtx.open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)?;
    let mut statistic_to_count = wtx.open_table(STATISTIC_TO_COUNT)?;

    if self.index.index_inscriptions || self.index.index_addresses || self.index.index_sats {
      self.index_utxo_entries(
        &block,
        txout_receiver,
        output_sender,
        utxo_cache,
        wtx,
        &mut inscription_id_to_sequence_number,
        &mut statistic_to_count,
        &mut sat_ranges_written,
        &mut outputs_in_block,
      )?;
    }

    if self.index.index_runes && self.height >= self.index.settings.first_rune_height() {
      let mut outpoint_to_rune_balances = wtx.open_table(OUTPOINT_TO_RUNE_BALANCES)?;
      let mut dune_id_to_rune_entry = wtx.open_table(RUNE_ID_TO_RUNE_ENTRY)?;
      let mut rune_to_dune_id = wtx.open_table(RUNE_TO_RUNE_ID)?;
      let mut sequence_number_to_dune_id = wtx.open_table(SEQUENCE_NUMBER_TO_RUNE_ID)?;
      let mut transaction_id_to_rune = wtx.open_table(TRANSACTION_ID_TO_RUNE)?;

      let dunes = statistic_to_count
        .get(&Statistic::Runes.into())?
        .map(|x| x.value())
        .unwrap_or(0);

      let mut dune_updater = DuneUpdater {
        event_sender: self.index.event_sender.as_ref(),
        block_time: block.header.time,
        burned: HashMap::new(),
        client: &self.index.client,
        height: self.height,
        id_to_entry: &mut dune_id_to_rune_entry,
        inscription_id_to_sequence_number: &mut inscription_id_to_sequence_number,
        minimum: Dune::minimum_at_height(
          self.index.settings.chain().network(),
          Height(self.height),
        ),
        outpoint_to_balances: &mut outpoint_to_rune_balances,
        rune_to_id: &mut rune_to_dune_id,
        dunes,
        sequence_number_to_dune_id: &mut sequence_number_to_dune_id,
        statistic_to_count: &mut statistic_to_count,
        transaction_id_to_rune: &mut transaction_id_to_rune,
      };

      for (i, (tx, txid)) in block.txdata.iter().enumerate() {
        dune_updater.index_runes(u32::try_from(i).unwrap(), tx, *txid)?;
      }

      dune_updater.update()?;
    }

    height_to_block_header.insert(&self.height, &block.header.store())?;

    self.height += 1;
    self.outputs_traversed += outputs_in_block;

    log::info!(
      "Wrote {sat_ranges_written} sat ranges from {outputs_in_block} outputs in {} ms",
      (Instant::now() - start).as_millis(),
    );

    Ok(())
  }

  fn index_utxo_entries<'wtx>(
    &mut self,
    block: &BlockData,
    txout_receiver: &mut broadcast::Receiver<TxOut>,
    output_sender: &mut mpsc::Sender<OutPoint>,
    utxo_cache: &mut HashMap<OutPoint, UtxoEntryBuf>,
    wtx: &'wtx WriteTransaction,
    inscription_id_to_sequence_number: &mut Table<'wtx, (u128, u128, u32), u32>,
    statistic_to_count: &mut Table<'wtx, u64, u64>,
    sat_ranges_written: &mut u64,
    outputs_in_block: &mut u64,
  ) -> Result {
    let mut collection_to_latest_child =
      wtx.open_table(COLLECTION_SEQUENCE_NUMBER_TO_LATEST_CHILD_SEQUENCE_NUMBER)?;
    let mut gallery_sequence_numbers = wtx.open_table(GALLERY_SEQUENCE_NUMBERS)?;
    let mut height_to_last_sequence_number = wtx.open_table(HEIGHT_TO_LAST_SEQUENCE_NUMBER)?;
    let mut home_inscriptions = wtx.open_table(HOME_INSCRIPTIONS)?;
    let mut inscription_number_to_sequence_number =
      wtx.open_table(INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER)?;
    let mut latest_child_to_collection =
      wtx.open_multimap_table(LATEST_CHILD_SEQUENCE_NUMBER_TO_COLLECTION_SEQUENCE_NUMBER)?;
    let mut outpoint_to_utxo_entry = wtx.open_table(OUTPOINT_TO_UTXO_ENTRY)?;
    let mut sat_to_satpoint = wtx.open_table(SAT_TO_SATPOINT)?;
    let mut koinu_to_sequence_number = wtx.open_multimap_table(SAT_TO_SEQUENCE_NUMBER)?;
    let mut script_pubkey_to_outpoint = wtx.open_multimap_table(SCRIPT_PUBKEY_TO_OUTPOINT)?;
    let mut sequence_number_to_children = wtx.open_multimap_table(SEQUENCE_NUMBER_TO_CHILDREN)?;
    let mut sequence_number_to_inscription_entry =
      wtx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;
    let mut transaction_id_to_transaction = wtx.open_table(TRANSACTION_ID_TO_TRANSACTION)?;

    let index_inscriptions = self.height >= self.index.settings.first_inscription_height()
      && self.index.index_inscriptions;

    // If the receiver still has inputs something went wrong in the last
    // block and we shouldn't recover from this and commit the last block
    if index_inscriptions {
      assert!(
        matches!(txout_receiver.try_recv(), Err(TryRecvError::Empty)),
        "Previous block did not consume all inputs"
      );
    }

    if !self.index.have_full_utxo_index() {
      // Send all missing input outpoints to be fetched
      let txids = block
        .txdata
        .iter()
        .map(|(_, txid)| txid)
        .collect::<HashSet<_>>();

      for (tx, _) in &block.txdata {
        for input in &tx.input {
          let prev_output = input.previous_output;
          // We don't need coinbase inputs
          if prev_output.is_null() {
            continue;
          }
          // We don't need inputs from txs earlier in the block, since
          // they'll be added to cache when the tx is indexed
          if txids.contains(&prev_output.txid) {
            continue;
          }
          // We don't need inputs we already have in our cache from earlier blocks
          if utxo_cache.contains_key(&prev_output) {
            continue;
          }
          // We don't need inputs we already have in our database
          if outpoint_to_utxo_entry.get(&prev_output.store())?.is_some() {
            continue;
          }
          // Send this outpoint to background thread to be fetched
          output_sender.blocking_send(prev_output)?;
        }
      }
    }

    let mut lost_koinu = statistic_to_count
      .get(&Statistic::LostSats.key())?
      .map(|lost_koinu| lost_koinu.value())
      .unwrap_or(0);

    let cursed_inscription_count = statistic_to_count
      .get(&Statistic::CursedInscriptions.key())?
      .map(|count| count.value())
      .unwrap_or(0);

    let blessed_inscription_count = statistic_to_count
      .get(&Statistic::BlessedInscriptions.key())?
      .map(|count| count.value())
      .unwrap_or(0);

    let unbound_inscriptions = statistic_to_count
      .get(&Statistic::UnboundInscriptions.key())?
      .map(|unbound_inscriptions| unbound_inscriptions.value())
      .unwrap_or(0);

    let next_sequence_number = sequence_number_to_inscription_entry
      .iter()?
      .next_back()
      .transpose()?
      .map(|(number, _id)| number.value() + 1)
      .unwrap_or(0);

    let home_inscription_count = home_inscriptions.len()?;

    let mut inscription_updater = InscriptionUpdater {
      blessed_inscription_count,
      collection_to_latest_child: &mut collection_to_latest_child,
      cursed_inscription_count,
      flotsam: Vec::new(),
      gallery_sequence_numbers: &mut gallery_sequence_numbers,
      height: self.height,
      home_inscription_count,
      home_inscriptions: &mut home_inscriptions,
      id_to_sequence_number: inscription_id_to_sequence_number,
      inscription_number_to_sequence_number: &mut inscription_number_to_sequence_number,
      latest_child_to_collection: &mut latest_child_to_collection,
      lost_koinu,
      next_sequence_number,
      reward: Height(self.height).subsidy(),
      koinu_to_sequence_number: &mut koinu_to_sequence_number,
      sequence_number_to_children: &mut sequence_number_to_children,
      sequence_number_to_entry: &mut sequence_number_to_inscription_entry,
      timestamp: block.header.time,
      transaction_buffer: Vec::new(),
      transaction_id_to_transaction: &mut transaction_id_to_transaction,
      unbound_inscriptions,
    };

    let mut coinbase_inputs = Vec::new();
    let mut lost_sat_ranges = Vec::new();

    if self.index.index_sats {
      let h = Height(self.height);
      if h.subsidy() > 0 {
        let start = h.starting_sat();
        coinbase_inputs.extend(SatRange::store((start.n(), (start + h.subsidy()).n())));
        self.sat_ranges_since_flush += 1;
      }
    }

    for (tx_offset, (tx, txid)) in block
      .txdata
      .iter()
      .enumerate()
      .skip(1)
      .chain(block.txdata.iter().enumerate().take(1))
    {
      log::trace!("Indexing transaction {tx_offset}…");

      let input_utxo_entries = if tx_offset == 0 {
        Vec::new()
      } else {
        tx.input
          .iter()
          .map(|input| {
            let outpoint = input.previous_output.store();

            let entry = if let Some(entry) = utxo_cache.remove(&OutPoint::load(outpoint)) {
              self.outputs_cached += 1;
              entry
            } else if let Some(entry) = outpoint_to_utxo_entry.remove(&outpoint)? {
              if self.index.index_addresses {
                let script_pubkey = entry.value().parse(self.index).script_pubkey();
                if !script_pubkey_to_outpoint.remove(script_pubkey, outpoint)? {
                  panic!("script pubkey entry ({script_pubkey:?}, {outpoint:?}) not found");
                }
              }

              entry.value().to_buf()
            } else {
              assert!(!self.index.have_full_utxo_index());
              let txout = txout_receiver.blocking_recv().map_err(|err| {
                anyhow!(
                  "failed to get transaction for {}: {err}",
                  input.previous_output
                )
              })?;

              let mut entry = UtxoEntryBuf::new();
              entry.push_value(txout.value.to_sat(), self.index);
              if self.index.index_addresses {
                entry.push_script_pubkey(txout.script_pubkey.as_bytes(), self.index);
              }

              entry
            };

            Ok(entry)
          })
          .collect::<Result<Vec<UtxoEntryBuf>>>()?
      };

      let input_utxo_entries = input_utxo_entries
        .iter()
        .map(|entry| entry.parse(self.index))
        .collect::<Vec<ParsedUtxoEntry>>();

      let mut output_utxo_entries = tx
        .output
        .iter()
        .map(|_| UtxoEntryBuf::new())
        .collect::<Vec<UtxoEntryBuf>>();

      let input_sat_ranges;
      if self.index.index_sats {
        let leftover_sat_ranges;

        if tx_offset == 0 {
          input_sat_ranges = Some(vec![coinbase_inputs.as_slice()]);
          leftover_sat_ranges = &mut lost_sat_ranges;
        } else {
          input_sat_ranges = Some(
            input_utxo_entries
              .iter()
              .map(|entry| entry.koinu_ranges())
              .collect(),
          );
          leftover_sat_ranges = &mut coinbase_inputs;
        }

        self.index_transaction_sats(
          tx,
          *txid,
          &mut sat_to_satpoint,
          &mut output_utxo_entries,
          input_sat_ranges.as_ref().unwrap(),
          leftover_sat_ranges,
          sat_ranges_written,
          outputs_in_block,
        )?;
      } else {
        input_sat_ranges = None;

        for (vout, txout) in tx.output.iter().enumerate() {
          output_utxo_entries[vout].push_value(txout.value.to_sat(), self.index);
        }
      }

      if self.index.index_addresses {
        self.index_transaction_output_script_pubkeys(tx, &mut output_utxo_entries);
      }

      if index_inscriptions {
        inscription_updater.index_inscriptions(
          tx,
          *txid,
          &input_utxo_entries,
          &mut output_utxo_entries,
          utxo_cache,
          self.index,
          input_sat_ranges.as_ref(),
        )?;

        self.index_dns_transaction(tx, *txid, block.header.time, wtx)?;
        self.index_drc20_transaction(tx, *txid, block.header.time, wtx)?;
      }

      for (vout, output_utxo_entry) in output_utxo_entries.into_iter().enumerate() {
        let vout = u32::try_from(vout).unwrap();
        utxo_cache.insert(OutPoint { txid: *txid, vout }, output_utxo_entry);
      }
    }

    if index_inscriptions {
      height_to_last_sequence_number
        .insert(&self.height, inscription_updater.next_sequence_number)?;
    }

    if !lost_sat_ranges.is_empty() {
      // Note that the lost-koinu outpoint is special, because (unlike real
      // outputs) it gets written to more than once.  commit() will merge
      // our new entry with any existing one.
      let utxo_entry = utxo_cache
        .entry(OutPoint::null())
        .or_insert(UtxoEntryBuf::empty(self.index));

      for chunk in lost_sat_ranges.chunks_exact(11) {
        let (start, end) = SatRange::load(chunk.try_into().unwrap());
        if !Koinu(start).common() {
          sat_to_satpoint.insert(
            &start,
            &KoinuPoint {
              outpoint: OutPoint::null(),
              offset: lost_koinu,
            }
            .store(),
          )?;
        }

        lost_koinu += end - start;
      }

      let mut new_utxo_entry = UtxoEntryBuf::new();
      new_utxo_entry.push_sat_ranges(&lost_sat_ranges, self.index);
      if self.index.index_addresses {
        new_utxo_entry.push_script_pubkey(&[], self.index);
      }

      *utxo_entry = UtxoEntryBuf::merged(utxo_entry, &new_utxo_entry, self.index);
    }

    statistic_to_count.insert(
      &Statistic::LostSats.key(),
      &if self.index.index_sats {
        lost_koinu
      } else {
        inscription_updater.lost_koinu
      },
    )?;

    statistic_to_count.insert(
      &Statistic::CursedInscriptions.key(),
      &inscription_updater.cursed_inscription_count,
    )?;

    statistic_to_count.insert(
      &Statistic::BlessedInscriptions.key(),
      &inscription_updater.blessed_inscription_count,
    )?;

    statistic_to_count.insert(
      &Statistic::UnboundInscriptions.key(),
      &inscription_updater.unbound_inscriptions,
    )?;

    Ok(())
  }

  fn index_transaction_output_script_pubkeys(
    &mut self,
    tx: &Transaction,
    output_utxo_entries: &mut [UtxoEntryBuf],
  ) {
    for (vout, txout) in tx.output.iter().enumerate() {
      output_utxo_entries[vout].push_script_pubkey(txout.script_pubkey.as_bytes(), self.index);
    }
  }

  fn index_transaction_sats(
    &mut self,
    tx: &Transaction,
    txid: Txid,
    sat_to_satpoint: &mut Table<u64, &KoinuPointValue>,
    output_utxo_entries: &mut [UtxoEntryBuf],
    input_sat_ranges: &[&[u8]],
    leftover_sat_ranges: &mut Vec<u8>,
    sat_ranges_written: &mut u64,
    outputs_traversed: &mut u64,
  ) -> Result {
    let mut pending_input_sat_range = None;
    let mut input_sat_ranges_iter = input_sat_ranges
      .iter()
      .flat_map(|slice| slice.chunks_exact(11));

    // Preallocate our temporary array, sized to hold the combined
    // sat ranges from our inputs.  We'll never need more than that
    // for a single output, even if we end up splitting some ranges.
    let mut koinu = Vec::with_capacity(
      input_sat_ranges
        .iter()
        .map(|slice| slice.len())
        .sum::<usize>(),
    );

    for (vout, output) in tx.output.iter().enumerate() {
      let outpoint = OutPoint {
        vout: vout.try_into().unwrap(),
        txid,
      };

      let mut remaining = output.value.to_sat();
      while remaining > 0 {
        let range = pending_input_sat_range.take().unwrap_or_else(|| {
          SatRange::load(
            input_sat_ranges_iter
              .next()
              .expect("insufficient inputs for transaction outputs")
              .try_into()
              .unwrap(),
          )
        });

        if !Koinu(range.0).common() {
          sat_to_satpoint.insert(
            &range.0,
            &KoinuPoint {
              outpoint,
              offset: output.value.to_sat() - remaining,
            }
            .store(),
          )?;
        }

        let count = range.1 - range.0;

        let assigned = if count > remaining {
          self.sat_ranges_since_flush += 1;
          let middle = range.0 + remaining;
          pending_input_sat_range = Some((middle, range.1));
          (range.0, middle)
        } else {
          range
        };

        koinu.extend_from_slice(&assigned.store());

        remaining -= assigned.1 - assigned.0;

        *sat_ranges_written += 1;
      }

      *outputs_traversed += 1;

      output_utxo_entries[vout].push_sat_ranges(&koinu, self.index);
      koinu.clear();
    }

    if let Some(range) = pending_input_sat_range {
      leftover_sat_ranges.extend(&range.store());
    }
    leftover_sat_ranges.extend(input_sat_ranges_iter.flatten());

    Ok(())
  }

  fn commit(
    &mut self,
    wtx: WriteTransaction,
    utxo_cache: HashMap<OutPoint, UtxoEntryBuf>,
  ) -> Result {
    log::info!(
      "Committing at block height {}, {} outputs traversed, {} in map, {} cached",
      self.height,
      self.outputs_traversed,
      utxo_cache.len(),
      self.outputs_cached
    );

    {
      let mut outpoint_to_utxo_entry = wtx.open_table(OUTPOINT_TO_UTXO_ENTRY)?;
      let mut script_pubkey_to_outpoint = wtx.open_multimap_table(SCRIPT_PUBKEY_TO_OUTPOINT)?;
      let mut sequence_number_to_satpoint = wtx.open_table(SEQUENCE_NUMBER_TO_SATPOINT)?;

      for (outpoint, mut utxo_entry) in utxo_cache {
        if Index::is_special_outpoint(outpoint)
          && let Some(old_entry) = outpoint_to_utxo_entry.get(&outpoint.store())?
        {
          utxo_entry = UtxoEntryBuf::merged(old_entry.value(), &utxo_entry, self.index);
        }

        outpoint_to_utxo_entry.insert(&outpoint.store(), utxo_entry.as_ref())?;

        let utxo_entry = utxo_entry.parse(self.index);
        if self.index.index_addresses {
          let script_pubkey = utxo_entry.script_pubkey();
          script_pubkey_to_outpoint.insert(script_pubkey, &outpoint.store())?;
        }

        if self.index.index_inscriptions {
          for (sequence_number, offset) in utxo_entry.parse_inscriptions() {
            let satpoint = KoinuPoint { outpoint, offset };
            sequence_number_to_satpoint.insert(sequence_number, &satpoint.store())?;
          }
        }
      }
    }

    Index::increment_statistic(&wtx, Statistic::OutputsTraversed, self.outputs_traversed)?;
    self.outputs_traversed = 0;
    Index::increment_statistic(&wtx, Statistic::SatRanges, self.sat_ranges_since_flush)?;
    self.sat_ranges_since_flush = 0;
    Index::increment_statistic(&wtx, Statistic::Commits, 1)?;
    wtx.commit()?;

    // Commit twice since due to a bug redb will only reuse pages freed in the
    // transaction before last.
    self.index.begin_write()?.commit()?;

    Reorg::update_savepoints(self.index, self.height)?;

    Ok(())
  }

  // ---------------------------------------------------------------------------
  // Dogecoin Name System (DNS) indexing
  // A DNS name is a plain-text inscription whose body is "name.namespace"
  // (e.g. "satoshi.doge"). The body must be valid UTF-8, contain exactly one
  // dot, and the part after the dot must be a known namespace.
  // ---------------------------------------------------------------------------
  fn index_dns_transaction(
    &mut self,
    tx: &Transaction,
    txid: Txid,
    block_time: u32,
    wtx: &WriteTransaction,
  ) -> Result<()> {
    use crate::subcommand::dns::is_valid_dns_namespace;

    let mut dns_name_to_entry = wtx.open_table(DNS_NAME_TO_ENTRY)?;
    let mut dns_inscription_id_to_name = wtx.open_table(DNS_INSCRIPTION_ID_TO_NAME)?;
    let mut dns_namespace_to_names = wtx.open_multimap_table(DNS_NAMESPACE_TO_NAMES)?;
    let mut statistic_to_count = wtx.open_table(STATISTIC_TO_COUNT)?;

    let envelopes = ParsedEnvelope::from_transaction(tx);

    for (envelope_index, envelope) in envelopes.into_iter().enumerate() {
      let Some(body) = envelope.payload.body() else {
        continue;
      };

      let Ok(text) = std::str::from_utf8(body) else {
        continue;
      };
      let name = text.trim();

      // Must be "label.namespace" — exactly one dot, non-empty parts
      let parts: Vec<&str> = name.split('.').collect();
      if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        continue;
      }
      let namespace = parts[1];
      if !is_valid_dns_namespace(namespace) {
        continue;
      }

      // Skip if name already registered (first inscription wins)
      if dns_name_to_entry.get(name)?.is_some() {
        continue;
      }

      let inscription_id = InscriptionId {
        txid,
        index: envelope_index as u32,
      };

      // Look up the sequence number for this inscription, then find its inscription number.
      // We copy the u32 out immediately so the table guard is released before the next lookup.
      let maybe_seq: Option<u32> = {
        let id_to_seq = wtx.open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)?;
        id_to_seq
          .get(&inscription_id.store())?
          .map(|g| g.value())
      };
      let inscription_number: i32 = if let Some(seq) = maybe_seq {
        let num_to_seq = wtx.open_table(INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER)?;
        num_to_seq
          .iter()?
          .filter_map(|r| r.ok())
          .find(|(_, s)| s.value() == seq)
          .map(|(n, _)| n.value())
          .unwrap_or(0)
      } else {
        0
      };

      let entry = DnsEntry {
        name: name.to_string(),
        owner_inscription_id: inscription_id,
        owner_inscription_number: inscription_number,
        height: self.height,
        timestamp: block_time,
        fee: 0, // fee calculation requires input lookup; leave 0 for now
        address: None, // resolved dynamically from current satpoint
        avatar: None,
        reverse: None,
      };

      dns_name_to_entry.insert(name, entry.store())?;
      dns_inscription_id_to_name.insert(&inscription_id.store(), name)?;
      dns_namespace_to_names.insert(namespace, name)?;

      // Update total count statistic
      let prev = statistic_to_count
        .get(&Statistic::DnsNames.key())?
        .map(|g| g.value())
        .unwrap_or(0);
      statistic_to_count.insert(&Statistic::DnsNames.key(), &(prev + 1))?;
    }

    Ok(())
  }

  // ---------------------------------------------------------------------------
  // DRC-20 token indexing
  //
  // Protocol: inscriptions with content-type text/plain or application/json,
  // body is JSON with {"p":"drc-20","op":"deploy|mint|transfer",...}.
  // Logic mirrors the bel-20-indexer (open source, Apache-2.0).
  // ---------------------------------------------------------------------------
  fn index_drc20_transaction(
    &mut self,
    tx: &Transaction,
    txid: Txid,
    block_time: u32,
    wtx: &WriteTransaction,
  ) -> Result<()> {
    use crate::subcommand::drc20::Drc20Transfer;

    // --- Step 1: Complete any pending transfers whose UTXOs are spent here ---
    {
      let mut outpoint_to_transfer = wtx.open_table(DRC20_OUTPOINT_TO_TRANSFER)?;
      let mut balance_table = wtx.open_table(DRC20_BALANCE)?;
      let mut transferable_table = wtx.open_table(DRC20_TRANSFERABLE)?;

      for input in &tx.input {
        let prev = input.previous_output.store();
        let maybe_transfer: Option<Drc20Transfer> = outpoint_to_transfer
          .get(&prev)?
          .and_then(|g| serde_json::from_slice(g.value()).ok());

        if let Some(transfer) = maybe_transfer {
          // Recipient = first output's address
          if let Some(out) = tx.output.first() {
            if let Some(recv_addr) = self
              .index
              .settings
              .chain()
              .address_string_from_script(out.script_pubkey.as_script())
            {
              // Credit recipient's available balance
              let recv_key = format!("{}\t{}", recv_addr, transfer.tick);
              let prev_bal: u128 = balance_table
                .get(recv_key.as_str())?
                .and_then(|g| serde_json::from_slice(g.value()).ok())
                .unwrap_or(0);
              let new_bal = serde_json::to_vec(&(prev_bal + transfer.amount))?;
              balance_table.insert(recv_key.as_str(), new_bal.as_slice())?;

              // Deduct from sender's transferable
              let send_key = format!("{}\t{}", transfer.from_address, transfer.tick);
              let prev_trf: u128 = transferable_table
                .get(send_key.as_str())?
                .and_then(|g| serde_json::from_slice(g.value()).ok())
                .unwrap_or(0);
              let new_trf = serde_json::to_vec(&prev_trf.saturating_sub(transfer.amount))?;
              transferable_table.insert(send_key.as_str(), new_trf.as_slice())?;
            }
          }
          outpoint_to_transfer.remove(&prev)?;
        }
      }
    }

    // --- Step 2: Scan envelopes in this transaction for DRC-20 ops ---
    let envelopes = ParsedEnvelope::from_transaction(tx);

    for (env_idx, envelope) in envelopes.into_iter().enumerate() {
      let Some(content_type) = envelope.payload.content_type() else {
        continue;
      };
      let Some(body) = envelope.payload.body() else {
        continue;
      };

      // Dogecoin wonky workaround: accept text/plain or application/json
      if !content_type.starts_with("text/plain")
        && !content_type.starts_with("application/json")
      {
        continue;
      }

      let Ok(text) = std::str::from_utf8(body) else {
        continue;
      };
      let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
        continue;
      };

      if json.get("p").and_then(|v| v.as_str()) != Some("drc-20") {
        continue;
      }
      let Some(op) = json.get("op").and_then(|v| v.as_str()) else {
        continue;
      };

      // Inscription destination address
      let out_idx = envelope.offset as usize;
      let Some(output) = tx.output.get(out_idx) else {
        continue;
      };
      let Some(addr) = self
        .index
        .settings
        .chain()
        .address_string_from_script(output.script_pubkey.as_script())
      else {
        continue;
      };

      let inscription_id = InscriptionId {
        txid,
        index: env_idx as u32,
      };

      match op {
        "deploy" => {
          self.drc20_deploy(&json, inscription_id, &addr, block_time, wtx)?;
        }
        "mint" => {
          self.drc20_mint(&json, &addr, wtx)?;
        }
        "transfer" => {
          let outpoint = OutPoint {
            txid,
            vout: out_idx as u32,
          };
          self.drc20_transfer_create(&json, &addr, outpoint, wtx)?;
        }
        _ => {}
      }
    }

    Ok(())
  }

  fn drc20_deploy(
    &mut self,
    json: &serde_json::Value,
    inscription_id: InscriptionId,
    deployer: &str,
    block_time: u32,
    wtx: &WriteTransaction,
  ) -> Result<()> {
    use crate::subcommand::drc20::{parse_amount, json_to_amount_str, Drc20Token};

    let Some(tick_raw) = json.get("tick").and_then(|v| v.as_str()) else {
      return Ok(());
    };
    if tick_raw.len() != 4 {
      return Ok(());
    }
    let tick_lower = tick_raw.to_lowercase();

    let mut token_table = wtx.open_table(DRC20_TICK_TO_TOKEN)?;

    // First deploy wins
    if token_table.get(tick_lower.as_str())?.is_some() {
      return Ok(());
    }

    let dec: u8 = json
      .get("dec")
      .and_then(|v| v.as_u64())
      .unwrap_or(8)
      .min(18) as u8;

    let Some(max_str) = json.get("max").and_then(|v| json_to_amount_str(v)) else {
      return Ok(());
    };
    let Some(max_supply) = parse_amount(&max_str, dec) else {
      return Ok(());
    };
    if max_supply == 0 {
      return Ok(());
    }

    let lim_str = json
      .get("lim")
      .and_then(|v| json_to_amount_str(v))
      .unwrap_or_else(|| max_str.clone());
    let Some(mint_limit) = parse_amount(&lim_str, dec) else {
      return Ok(());
    };
    if mint_limit == 0 {
      return Ok(());
    }

    let token = Drc20Token {
      tick: tick_raw.to_string(),
      max_supply,
      mint_limit,
      decimals: dec,
      minted: 0,
      deploy_inscription: inscription_id.to_string(),
      deploy_height: self.height,
      deploy_timestamp: block_time,
      deployer: deployer.to_string(),
      mint_count: 0,
    };

    let bytes = serde_json::to_vec(&token)?;
    token_table.insert(tick_lower.as_str(), bytes.as_slice())?;

    // Update token count statistic
    let mut statistic_to_count = wtx.open_table(STATISTIC_TO_COUNT)?;
    let prev = statistic_to_count
      .get(&Statistic::Drc20Tokens.key())?
      .map(|g| g.value())
      .unwrap_or(0);
    statistic_to_count.insert(&Statistic::Drc20Tokens.key(), &(prev + 1))?;

    Ok(())
  }

  fn drc20_mint(
    &mut self,
    json: &serde_json::Value,
    recipient: &str,
    wtx: &WriteTransaction,
  ) -> Result<()> {
    use crate::subcommand::drc20::{parse_amount, json_to_amount_str, Drc20Token};

    let Some(tick_raw) = json.get("tick").and_then(|v| v.as_str()) else {
      return Ok(());
    };
    if tick_raw.len() != 4 {
      return Ok(());
    }
    let tick_lower = tick_raw.to_lowercase();

    let mut token_table = wtx.open_table(DRC20_TICK_TO_TOKEN)?;
    let Some(token_guard) = token_table.get(tick_lower.as_str())? else {
      return Ok(()); // tick not deployed
    };
    let mut token: Drc20Token = serde_json::from_slice(token_guard.value())?;
    drop(token_guard);

    if token.minted >= token.max_supply {
      return Ok(()); // fully minted
    }

    let Some(amt_str) = json.get("amt").and_then(|v| json_to_amount_str(v)) else {
      return Ok(());
    };
    let Some(mut amt) = parse_amount(&amt_str, token.decimals) else {
      return Ok(());
    };
    if amt == 0 {
      return Ok(());
    }

    // Enforce per-mint limit
    if amt > token.mint_limit {
      return Ok(());
    }

    // Cap at remaining supply
    let remaining = token.max_supply - token.minted;
    if amt > remaining {
      amt = remaining;
    }

    // Credit recipient's available balance
    let bal_key = format!("{}\t{}", recipient, tick_lower);
    let mut balance_table = wtx.open_table(DRC20_BALANCE)?;
    let prev: u128 = balance_table
      .get(bal_key.as_str())?
      .and_then(|g| serde_json::from_slice(g.value()).ok())
      .unwrap_or(0);
    let new_bal = serde_json::to_vec(&(prev + amt))?;
    balance_table.insert(bal_key.as_str(), new_bal.as_slice())?;

    // Update token metadata
    token.minted += amt;
    token.mint_count += 1;
    let updated = serde_json::to_vec(&token)?;
    token_table.insert(tick_lower.as_str(), updated.as_slice())?;

    Ok(())
  }

  fn drc20_transfer_create(
    &mut self,
    json: &serde_json::Value,
    sender: &str,
    outpoint: OutPoint,
    wtx: &WriteTransaction,
  ) -> Result<()> {
    use crate::subcommand::drc20::{parse_amount, json_to_amount_str, Drc20Transfer, Drc20Token};

    let Some(tick_raw) = json.get("tick").and_then(|v| v.as_str()) else {
      return Ok(());
    };
    if tick_raw.len() != 4 {
      return Ok(());
    }
    let tick_lower = tick_raw.to_lowercase();

    let token_table = wtx.open_table(DRC20_TICK_TO_TOKEN)?;
    let Some(token_guard) = token_table.get(tick_lower.as_str())? else {
      return Ok(());
    };
    let token: Drc20Token = serde_json::from_slice(token_guard.value())?;
    drop(token_guard);
    drop(token_table);

    let Some(amt_str) = json.get("amt").and_then(|v| json_to_amount_str(v)) else {
      return Ok(());
    };
    let Some(amt) = parse_amount(&amt_str, token.decimals) else {
      return Ok(());
    };
    if amt == 0 {
      return Ok(());
    }

    // Check sender has enough available balance
    let bal_key = format!("{}\t{}", sender, tick_lower);
    let mut balance_table = wtx.open_table(DRC20_BALANCE)?;
    let avail: u128 = balance_table
      .get(bal_key.as_str())?
      .and_then(|g| serde_json::from_slice(g.value()).ok())
      .unwrap_or(0);

    if amt > avail {
      return Ok(()); // insufficient balance
    }

    // Deduct from available, add to transferable
    let new_avail = serde_json::to_vec(&(avail - amt))?;
    balance_table.insert(bal_key.as_str(), new_avail.as_slice())?;

    let mut transferable_table = wtx.open_table(DRC20_TRANSFERABLE)?;
    let prev_trf: u128 = transferable_table
      .get(bal_key.as_str())?
      .and_then(|g| serde_json::from_slice(g.value()).ok())
      .unwrap_or(0);
    let new_trf = serde_json::to_vec(&(prev_trf + amt))?;
    transferable_table.insert(bal_key.as_str(), new_trf.as_slice())?;

    // Record this outpoint as a pending transfer
    let transfer = Drc20Transfer {
      tick: tick_lower,
      amount: amt,
      from_address: sender.to_string(),
    };
    let transfer_bytes = serde_json::to_vec(&transfer)?;
    let mut outpoint_to_transfer = wtx.open_table(DRC20_OUTPOINT_TO_TRANSFER)?;
    outpoint_to_transfer.insert(&outpoint.store(), transfer_bytes.as_slice())?;

    Ok(())
  }
}
