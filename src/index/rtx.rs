use super::*;

pub(crate) struct Rtx(pub(crate) redb::ReadTransaction);

impl Rtx {
  pub(crate) fn block_height(&self) -> Result<Option<Height>> {
    Ok(
      self
        .0
        .open_table(HEIGHT_TO_BLOCK_HEADER)?
        .range(0..)?
        .next_back()
        .transpose()?
        .map(|(height, _header)| Height(height.value())),
    )
  }

  pub(crate) fn block_count(&self) -> Result<u32> {
    Ok(
      self
        .0
        .open_table(HEIGHT_TO_BLOCK_HEADER)?
        .range(0..)?
        .next_back()
        .transpose()?
        .map(|(height, _header)| height.value() + 1)
        .unwrap_or(0),
    )
  }

  fn block_hash_at(&self, h: u32) -> Result<Option<BlockHash>> {
    let height_to_block_hash = self.0.open_table(HEIGHT_TO_BLOCK_HASH)?;
    let height_to_block_header = self.0.open_table(HEIGHT_TO_BLOCK_HEADER)?;

    if let Some(guard) = height_to_block_hash.get(h)? {
      return Ok(Some(BlockHash::from_byte_array(*guard.value())));
    }
    Ok(
      height_to_block_header
        .get(h)?
        .map(|header| Header::load(*header.value()).block_hash()),
    )
  }

  pub(crate) fn block_hash(&self, height: Option<u32>) -> Result<Option<BlockHash>> {
    let h = match height {
      Some(h) => h,
      None => {
        let height_to_block_header = self.0.open_table(HEIGHT_TO_BLOCK_HEADER)?;
        let Some((height, _)) = height_to_block_header.range(0..)?.next_back().transpose()? else {
          return Ok(None);
        };
        return self.block_hash_at(height.value());
      }
    };

    self.block_hash_at(h)
  }
}
