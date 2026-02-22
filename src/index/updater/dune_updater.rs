use super::*;

pub(super) struct DuneUpdater<'a, 'tx, 'client> {
  pub(super) block_time: u32,
  pub(super) burned: HashMap<DuneId, Lot>,
  pub(super) client: &'client Client,
  pub(super) event_sender: Option<&'a mpsc::Sender<Event>>,
  pub(super) height: u32,
  pub(super) id_to_entry: &'a mut Table<'tx, DuneIdValue, DuneEntryValue>,
  pub(super) inscription_id_to_sequence_number: &'a Table<'tx, InscriptionIdValue, u32>,
  pub(super) minimum: Dune,
  pub(super) outpoint_to_balances: &'a mut Table<'tx, &'static OutPointValue, &'static [u8]>,
  pub(super) rune_to_id: &'a mut Table<'tx, u128, DuneIdValue>,
  pub(super) dunes: u64,
  pub(super) sequence_number_to_dune_id: &'a mut Table<'tx, u32, DuneIdValue>,
  pub(super) statistic_to_count: &'a mut Table<'tx, u64, u64>,
  pub(super) transaction_id_to_rune: &'a mut Table<'tx, &'static TxidValue, u128>,
}

impl DuneUpdater<'_, '_, '_> {
  pub(super) fn index_runes(&mut self, tx_index: u32, tx: &Transaction, txid: Txid) -> Result<()> {
    let artifact = Dunestone::decipher(tx);

    let mut unallocated = self.unallocated(tx)?;

    let mut allocated: Vec<HashMap<DuneId, Lot>> = vec![HashMap::new(); tx.output.len()];

    if let Some(artifact) = &artifact {
      if let Some(id) = artifact.mint()
        && let Some(amount) = self.mint(id)?
      {
        *unallocated.entry(id).or_default() += amount;

        if let Some(sender) = self.event_sender {
          sender.blocking_send(Event::RuneMinted {
            block_height: self.height,
            txid,
            dune_id: id,
            amount: amount.n(),
          })?;
        }
      }

      let etched = self.etched(tx_index, tx, artifact)?;

      if let Artifact::Dunestone(dunestone) = artifact {
        if let Some((id, ..)) = etched {
          *unallocated.entry(id).or_default() +=
            dunestone.etching.unwrap().premine.unwrap_or_default();
        }

        for Edict { id, amount, output } in dunestone.edicts.iter().copied() {
          let amount = Lot(amount);

          // edicts with output values greater than the number of outputs
          // should never be produced by the edict parser
          let output = usize::try_from(output).unwrap();
          assert!(output <= tx.output.len());

          let id = if id == DuneId::default() {
            let Some((id, ..)) = etched else {
              continue;
            };

            id
          } else {
            id
          };

          let Some(balance) = unallocated.get_mut(&id) else {
            continue;
          };

          let mut allocate = |balance: &mut Lot, amount: Lot, output: usize| {
            if amount > 0 {
              *balance -= amount;
              *allocated[output].entry(id).or_default() += amount;
            }
          };

          if output == tx.output.len() {
            // find non-OP_RETURN outputs
            let destinations = tx
              .output
              .iter()
              .enumerate()
              .filter_map(|(output, tx_out)| {
                (!tx_out.script_pubkey.is_op_return()).then_some(output)
              })
              .collect::<Vec<usize>>();

            if !destinations.is_empty() {
              if amount == 0 {
                // if amount is zero, divide balance between eligible outputs
                let amount = *balance / destinations.len() as u128;
                let remainder = usize::try_from(*balance % destinations.len() as u128).unwrap();

                for (i, output) in destinations.iter().enumerate() {
                  allocate(
                    balance,
                    if i < remainder { amount + 1 } else { amount },
                    *output,
                  );
                }
              } else {
                // if amount is non-zero, distribute amount to eligible outputs
                for output in destinations {
                  allocate(balance, amount.min(*balance), output);
                }
              }
            }
          } else {
            // Get the allocatable amount
            let amount = if amount == 0 {
              *balance
            } else {
              amount.min(*balance)
            };

            allocate(balance, amount, output);
          }
        }
      }

      if let Some((id, dune)) = etched {
        self.create_rune_entry(txid, artifact, id, dune)?;
      }
    }

    let mut burned: HashMap<DuneId, Lot> = HashMap::new();

    if let Some(Artifact::Cenotaph(_)) = artifact {
      for (id, balance) in unallocated {
        *burned.entry(id).or_default() += balance;
      }
    } else {
      let pointer = artifact
        .map(|artifact| match artifact {
          Artifact::Dunestone(dunestone) => dunestone.pointer,
          Artifact::Cenotaph(_) => unreachable!(),
        })
        .unwrap_or_default();

      // assign all un-allocated dunes to the default output, or the first non
      // OP_RETURN output if there is no default
      if let Some(vout) = pointer
        .map(|pointer| pointer.into_usize())
        .inspect(|&pointer| assert!(pointer < allocated.len()))
        .or_else(|| {
          tx.output
            .iter()
            .enumerate()
            .find(|(_vout, tx_out)| !tx_out.script_pubkey.is_op_return())
            .map(|(vout, _tx_out)| vout)
        })
      {
        for (id, balance) in unallocated {
          if balance > 0 {
            *allocated[vout].entry(id).or_default() += balance;
          }
        }
      } else {
        for (id, balance) in unallocated {
          if balance > 0 {
            *burned.entry(id).or_default() += balance;
          }
        }
      }
    }

    // update outpoint balances
    let mut buffer: Vec<u8> = Vec::new();
    for (vout, balances) in allocated.into_iter().enumerate() {
      if balances.is_empty() {
        continue;
      }

      // increment burned balances
      if tx.output[vout].script_pubkey.is_op_return() {
        for (id, balance) in &balances {
          *burned.entry(*id).or_default() += *balance;
        }
        continue;
      }

      buffer.clear();

      let mut balances = balances.into_iter().collect::<Vec<(DuneId, Lot)>>();

      // Sort balances by id so tests can assert balances in a fixed order
      balances.sort();

      let outpoint = OutPoint {
        txid,
        vout: vout.try_into().unwrap(),
      };

      for (id, balance) in balances {
        Index::encode_rune_balance(id, balance.n(), &mut buffer);

        if let Some(sender) = self.event_sender {
          sender.blocking_send(Event::RuneTransferred {
            outpoint,
            block_height: self.height,
            txid,
            dune_id: id,
            amount: balance.0,
          })?;
        }
      }

      self
        .outpoint_to_balances
        .insert(&outpoint.store(), buffer.as_slice())?;
    }

    // increment entries with burned dunes
    for (id, amount) in burned {
      *self.burned.entry(id).or_default() += amount;

      if let Some(sender) = self.event_sender {
        sender.blocking_send(Event::RuneBurned {
          block_height: self.height,
          txid,
          dune_id: id,
          amount: amount.n(),
        })?;
      }
    }

    Ok(())
  }

  pub(super) fn update(self) -> Result {
    for (dune_id, burned) in self.burned {
      let mut entry = DuneEntry::load(self.id_to_entry.get(&dune_id.store())?.unwrap().value());
      entry.burned = entry.burned.checked_add(burned.n()).unwrap();
      self.id_to_entry.insert(&dune_id.store(), entry.store())?;
    }

    Ok(())
  }

  fn create_rune_entry(
    &mut self,
    txid: Txid,
    artifact: &Artifact,
    id: DuneId,
    dune: Dune,
  ) -> Result {
    self.rune_to_id.insert(dune.store(), id.store())?;
    self
      .transaction_id_to_rune
      .insert(&txid.store(), dune.store())?;

    let number = self.dunes;
    self.dunes += 1;

    self
      .statistic_to_count
      .insert(&Statistic::Runes.into(), self.dunes)?;

    let entry = match artifact {
      Artifact::Cenotaph(_) => DuneEntry {
        block: id.block,
        burned: 0,
        divisibility: 0,
        etching: txid,
        terms: None,
        mints: 0,
        number,
        premine: 0,
        spaced_dune: SpacedDune { dune, spacers: 0 },
        symbol: None,
        timestamp: self.block_time.into(),
        turbo: false,
      },
      Artifact::Dunestone(Dunestone { etching, .. }) => {
        let Etching {
          divisibility,
          terms,
          premine,
          spacers,
          symbol,
          turbo,
          ..
        } = etching.unwrap();

        DuneEntry {
          block: id.block,
          burned: 0,
          divisibility: divisibility.unwrap_or_default(),
          etching: txid,
          terms,
          mints: 0,
          number,
          premine: premine.unwrap_or_default(),
          spaced_dune: SpacedDune {
            dune,
            spacers: spacers.unwrap_or_default(),
          },
          symbol,
          timestamp: self.block_time.into(),
          turbo,
        }
      }
    };

    self.id_to_entry.insert(id.store(), entry.store())?;

    if let Some(sender) = self.event_sender {
      sender.blocking_send(Event::RuneEtched {
        block_height: self.height,
        txid,
        dune_id: id,
      })?;
    }

    let inscription_id = InscriptionId { txid, index: 0 };

    if let Some(sequence_number) = self
      .inscription_id_to_sequence_number
      .get(&inscription_id.store())?
    {
      self
        .sequence_number_to_dune_id
        .insert(sequence_number.value(), id.store())?;
    }

    Ok(())
  }

  fn etched(
    &mut self,
    tx_index: u32,
    tx: &Transaction,
    artifact: &Artifact,
  ) -> Result<Option<(DuneId, Dune)>> {
    let dune = match artifact {
      Artifact::Dunestone(dunestone) => match dunestone.etching {
        Some(etching) => etching.dune,
        None => return Ok(None),
      },
      Artifact::Cenotaph(cenotaph) => match cenotaph.etching {
        Some(dune) => Some(dune),
        None => return Ok(None),
      },
    };

    let dune = if let Some(dune) = dune {
      if dune < self.minimum
        || dune.is_reserved()
        || self.rune_to_id.get(dune.0)?.is_some()
        || !self.tx_commits_to_rune(tx, dune)?
      {
        return Ok(None);
      }
      dune
    } else {
      let reserved_runes = self
        .statistic_to_count
        .get(&Statistic::ReservedRunes.into())?
        .map(|entry| entry.value())
        .unwrap_or_default();

      self
        .statistic_to_count
        .insert(&Statistic::ReservedRunes.into(), reserved_runes + 1)?;

      Dune::reserved(self.height.into(), tx_index)
    };

    Ok(Some((
      DuneId {
        block: self.height.into(),
        tx: tx_index,
      },
      dune,
    )))
  }

  fn mint(&mut self, id: DuneId) -> Result<Option<Lot>> {
    let Some(entry) = self.id_to_entry.get(&id.store())? else {
      return Ok(None);
    };

    let mut rune_entry = DuneEntry::load(entry.value());

    let Ok(amount) = rune_entry.mintable(self.height.into()) else {
      return Ok(None);
    };

    drop(entry);

    rune_entry.mints += 1;

    self.id_to_entry.insert(&id.store(), rune_entry.store())?;

    Ok(Some(Lot(amount)))
  }

  fn tx_commits_to_rune(&self, tx: &Transaction, dune: Dune) -> Result<bool> {
    let commitment = dune.commitment();

    for input in &tx.input {
      // extracting a tapscript does not indicate that the input being spent
      // was actually a taproot output. this is checked below, when we load the
      // output's entry from the database
      let Some(tapscript) = unversioned_leaf_script_from_witness(&input.witness) else {
        continue;
      };

      for instruction in tapscript.instructions() {
        // ignore errors, since the extracted script may not be valid
        let Ok(instruction) = instruction else {
          break;
        };

        let Some(pushbytes) = instruction.push_bytes() else {
          continue;
        };

        if pushbytes.as_bytes() != commitment {
          continue;
        }

        let Some(tx_info) = self
          .client
          .get_raw_transaction_info(&input.previous_output.txid, None)
          .into_option()?
        else {
          panic!(
            "can't get input transaction: {}",
            input.previous_output.txid
          );
        };

        let taproot = tx_info.vout[input.previous_output.vout.into_usize()]
          .script_pub_key
          .script()?
          .is_p2tr();

        if !taproot {
          continue;
        }

        let commit_tx_height = self
          .client
          .get_block_header_info(&tx_info.blockhash.unwrap())
          .into_option()?
          .unwrap()
          .height;

        let confirmations = self
          .height
          .checked_sub(commit_tx_height.try_into().unwrap())
          .unwrap()
          + 1;

        if confirmations >= u32::from(Dunestone::COMMIT_CONFIRMATIONS) {
          return Ok(true);
        }
      }
    }

    Ok(false)
  }

  fn unallocated(&mut self, tx: &Transaction) -> Result<HashMap<DuneId, Lot>> {
    // map of dune ID to un-allocated balance of that dune
    let mut unallocated: HashMap<DuneId, Lot> = HashMap::new();

    // increment unallocated dunes with the dunes in tx inputs
    for input in &tx.input {
      if let Some(guard) = self
        .outpoint_to_balances
        .remove(&input.previous_output.store())?
      {
        let buffer = guard.value();
        let mut i = 0;
        while i < buffer.len() {
          let ((id, balance), len) = Index::decode_rune_balance(&buffer[i..]).unwrap();
          i += len;
          *unallocated.entry(id).or_default() += balance;
        }
      }
    }

    Ok(unallocated)
  }
}
