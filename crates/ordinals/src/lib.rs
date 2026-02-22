//! Types for interoperating with ordinals, inscriptions, and dunes.
#![allow(clippy::large_enum_variant)]

use {
  bitcoin::{
    Network, OutPoint, ScriptBuf, Transaction,
    consensus::{Decodable, Encodable},

    opcodes,
    script::{self, Instruction},
  },
  derive_more::{Display, FromStr},
  serde::{Deserialize, Serialize},
  serde_with::{DeserializeFromStr, SerializeDisplay},
  std::{
    sync::LazyLock,
    cmp,
    collections::{HashMap, VecDeque},
    fmt::{self, Formatter},
    num::ParseIntError,
    ops::{Add, AddAssign, Sub},
  },
  thiserror::Error,
};

pub use {
  artifact::Artifact, cenotaph::Cenotaph, charm::Charm, decimal_koinu::DecimalKoinu, degree::Degree,
  edict::Edict, epoch::Epoch, etching::Etching, flaw::Flaw, height::Height, pile::Pile,
  rarity::Rarity, dune::Dune, dune_id::DuneId, dunestone::Dunestone, koinu::Koinu, koinu_point::KoinuPoint,
  spaced_dune::SpacedDune, terms::Terms,
};

pub const COIN_VALUE: u64 = 100_000_000;
pub const CYCLE_EPOCHS: u32 = 6;

// Dogecoin-specific chain constants.
pub const DIFFCHANGE_INTERVAL: u32 = 1;
pub const SUBSIDY_HALVING_INTERVAL: u32 = 1;

fn default<T: Default>() -> T {
  Default::default()
}

mod artifact;
mod cenotaph;
mod charm;
mod decimal_koinu;
mod degree;
mod edict;
mod epoch;
mod etching;
mod flaw;
mod height;
mod pile;
mod rarity;
mod dune;
mod dune_id;
mod dunestone;
pub mod koinu;
pub mod koinu_point;
pub mod spaced_dune;
mod terms;
pub mod varint;
