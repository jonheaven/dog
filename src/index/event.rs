use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
  InscriptionCreated {
    block_height: u32,
    charms: u16,
    inscription_id: InscriptionId,
    location: Option<KoinuPoint>,
    parent_inscription_ids: Vec<InscriptionId>,
    sequence_number: u32,
  },
  InscriptionTransferred {
    block_height: u32,
    inscription_id: InscriptionId,
    new_location: KoinuPoint,
    old_location: KoinuPoint,
    sequence_number: u32,
  },
  RuneBurned {
    amount: u128,
    block_height: u32,
    dune_id: DuneId,
    txid: Txid,
  },
  RuneEtched {
    block_height: u32,
    dune_id: DuneId,
    txid: Txid,
  },
  RuneMinted {
    amount: u128,
    block_height: u32,
    dune_id: DuneId,
    txid: Txid,
  },
  RuneTransferred {
    amount: u128,
    block_height: u32,
    outpoint: OutPoint,
    dune_id: DuneId,
    txid: Txid,
  },
}
