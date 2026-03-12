use {
  super::*,
  serde_hex::{SerHex, Strict},
};

pub use crate::{
  subcommand::decode::RawOutput as Decode,
  templates::{
    BlocksHtml as Blocks, DuneHtml as Dune, DunesHtml as Dunes, StatusHtml as Status,
    TransactionHtml as Transaction,
  },
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Block {
  pub best_height: u32,
  pub hash: BlockHash,
  pub height: u32,
  pub inscriptions: Vec<InscriptionId>,
  pub dunes: Vec<SpacedDune>,
  pub target: BlockHash,
  pub transactions: Vec<bitcoin::blockdata::transaction::Transaction>,
}

impl Block {
  pub(crate) fn new(
    block: bitcoin::Block,
    height: Height,
    best_height: Height,
    inscriptions: Vec<InscriptionId>,
    dunes: Vec<SpacedDune>,
  ) -> Self {
    Self {
      hash: block.header.block_hash(),
      target: target_as_block_hash(block.header.target()),
      height: height.0,
      best_height: best_height.0,
      inscriptions,
      dunes,
      transactions: block.txdata,
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockInfo {
  pub average_fee: u64,
  pub average_fee_rate: u64,
  pub bits: u32,
  #[serde(with = "SerHex::<Strict>")]
  pub chainwork: [u8; 32],
  pub confirmations: i32,
  pub difficulty: f64,
  pub hash: BlockHash,
  pub feerate_percentiles: [u64; 5],
  pub height: u32,
  pub max_fee: u64,
  pub max_fee_rate: u64,
  pub max_tx_size: u32,
  pub median_fee: u64,
  pub median_time: Option<u64>,
  pub merkle_root: TxMerkleNode,
  pub min_fee: u64,
  pub min_fee_rate: u64,
  pub next_block: Option<BlockHash>,
  pub nonce: u32,
  pub previous_block: Option<BlockHash>,
  pub subsidy: u64,
  pub target: BlockHash,
  pub timestamp: u64,
  pub total_fee: u64,
  pub total_size: usize,
  pub total_weight: usize,
  pub transaction_count: u64,
  pub version: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Children {
  pub ids: Vec<InscriptionId>,
  pub more: bool,
  pub page: usize,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ChildInscriptions {
  pub children: Vec<RelativeInscriptionRecursive>,
  pub more: bool,
  pub page: usize,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ParentInscriptions {
  pub parents: Vec<RelativeInscriptionRecursive>,
  pub more: bool,
  pub page: usize,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Inscription {
  pub address: Option<String>,
  pub charms: Vec<Charm>,
  pub child_count: u64,
  pub children: Vec<InscriptionId>,
  pub content_length: Option<usize>,
  pub content_type: Option<String>,
  pub effective_content_type: Option<String>,
  pub fee: u64,
  pub height: u32,
  pub id: InscriptionId,
  pub metaprotocol: Option<String>,
  pub next: Option<InscriptionId>,
  pub number: i32,
  pub parents: Vec<InscriptionId>,
  pub previous: Option<InscriptionId>,
  pub properties: Properties,
  pub dune: Option<SpacedDune>,
  pub sat: Option<doginals::Koinu>,
  pub satpoint: KoinuPoint,
  pub timestamp: i64,
  pub value: Option<u64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InscriptionRecursive {
  pub charms: Vec<Charm>,
  pub content_type: Option<String>,
  pub content_length: Option<usize>,
  pub delegate: Option<InscriptionId>,
  pub fee: u64,
  pub height: u32,
  pub id: InscriptionId,
  pub number: i32,
  pub output: OutPoint,
  pub sat: Option<doginals::Koinu>,
  pub satpoint: KoinuPoint,
  pub timestamp: i64,
  pub value: Option<u64>,
  pub address: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RelativeInscriptionRecursive {
  pub charms: Vec<Charm>,
  pub fee: u64,
  pub height: u32,
  pub id: InscriptionId,
  pub number: i32,
  pub output: OutPoint,
  pub sat: Option<doginals::Koinu>,
  pub satpoint: KoinuPoint,
  pub timestamp: i64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Inscriptions {
  pub ids: Vec<InscriptionId>,
  pub more: bool,
  pub page_index: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct UtxoRecursive {
  pub inscriptions: Option<Vec<InscriptionId>>,
  pub dunes: Option<BTreeMap<SpacedDune, Pile>>,
  pub koinu_ranges: Option<Vec<(u64, u64)>>,
  pub value: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Output {
  pub address: Option<Address<NetworkUnchecked>>,
  pub confirmations: u32,
  pub indexed: bool,
  pub inscriptions: Option<Vec<InscriptionId>>,
  pub outpoint: OutPoint,
  pub dunes: Option<BTreeMap<SpacedDune, Pile>>,
  pub koinu_ranges: Option<Vec<(u64, u64)>>,
  pub script_pubkey: ScriptBuf,
  pub spent: bool,
  pub transaction: Txid,
  pub value: u64,
}

impl Output {
  pub fn new(
    chain: Chain,
    confirmations: u32,
    inscriptions: Option<Vec<InscriptionId>>,
    outpoint: OutPoint,
    tx_out: TxOut,
    indexed: bool,
    dunes: Option<BTreeMap<SpacedDune, Pile>>,
    koinu_ranges: Option<Vec<(u64, u64)>>,
    spent: bool,
  ) -> Self {
    Self {
      address: chain
        .address_from_script(&tx_out.script_pubkey)
        .ok()
        .map(|address| uncheck(&address)),
      confirmations,
      indexed,
      inscriptions,
      outpoint,
      dunes,
      koinu_ranges,
      script_pubkey: tx_out.script_pubkey,
      spent,
      transaction: outpoint.txid,
      value: tx_out.value.to_sat(),
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Koinu {
  pub address: Option<String>,
  pub block: u32,
  pub charms: Vec<Charm>,
  pub cycle: u32,
  pub decimal: String,
  pub degree: String,
  pub epoch: u32,
  pub inscriptions: Vec<InscriptionId>,
  pub name: String,
  pub number: u64,
  pub offset: u64,
  pub percentile: String,
  pub period: u32,
  pub rarity: Rarity,
  pub satpoint: Option<KoinuPoint>,
  pub timestamp: i64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SatInscription {
  pub id: Option<InscriptionId>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SatInscriptions {
  pub ids: Vec<InscriptionId>,
  pub more: bool,
  pub page: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AddressInfo {
  pub outputs: Vec<OutPoint>,
  pub inscriptions: Option<Vec<InscriptionId>>,
  pub sat_balance: u64,
  pub dunes_balances: Option<Vec<(SpacedDune, Decimal, Option<char>)>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lazy_lookup: Option<bool>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Offers {
  pub offers: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct HealthJson {
  pub index_tip: u32,
  pub chain_tip: u32,
  pub lag_blocks: u32,
  pub status: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LiveStatusJson {
  pub chain: Chain,
  pub height: Option<u32>,
  pub chain_tip: u32,
  pub lag_blocks: u32,
  pub status: String,
  pub syncing: bool,
  pub blocks_per_second: f64,
  pub inscriptions_per_second: f64,
  pub inscriptions: u64,
  pub dunes: u64,
  pub dogemaps: u64,
  pub dogespells: u64,
  pub dmp: u64,
  pub dogelotto: u64,
  pub active_protocols: Vec<String>,
  pub updated_at: i64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MonitorStatsJson {
  pub total_indexed: u64,
  pub blessed_inscriptions: u64,
  pub cursed_inscriptions: u64,
  pub memory_usage_bytes: u64,
  pub reorg_count: u64,
  pub webhook_deliveries: u64,
  pub initial_sync_seconds: u64,
  pub uptime_seconds: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MonitorFeedItem {
  pub kind: String,
  pub title: String,
  pub subtitle: String,
  pub link: String,
  pub height: Option<u32>,
  pub timestamp: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MonitorJson {
  pub status: LiveStatusJson,
  pub stats: MonitorStatsJson,
  pub feed: Vec<MonitorFeedItem>,
}

/// Response for `/r/txproof/{txid}` — merkle inclusion proof for a confirmed transaction.
/// All hash values are in display (reversed) byte order, matching Bitcoin/Dogecoin RPC convention.
/// The `proof` array contains sibling hashes at each level of the merkle tree,
/// prefixed with "L-" (sibling is on the left) or "r-" (sibling is on the right).
#[derive(Debug, Serialize, Deserialize)]
pub struct TxProof {
  pub txid: String,
  pub blockhash: String,
  pub merkleroot: String,
  pub time: u64,
  pub height: u32,
  pub proof: Vec<String>,
}
