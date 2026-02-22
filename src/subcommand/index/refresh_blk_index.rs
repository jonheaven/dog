use {
  super::super::*,
  crate::index::updater::blk_reader::refresh_index_copy,
};

pub(crate) fn run(settings: Settings) -> SubcommandResult {
  let Some(blocks_dir) = settings.dogecoin_blocks_dir() else {
    eprintln!("Could not determine Dogecoin blocks directory.");
    eprintln!("Set --dogecoin-data-dir or DOGECOIN_DATA_DIR.");
    return Ok(None);
  };

  let live_index = blocks_dir.join("index");
  if !live_index.exists() {
    eprintln!("Block index not found at {}", live_index.display());
    eprintln!("Is Dogecoin Core installed and has it synced?");
    return Ok(None);
  }

  let copy_dir = settings.data_dir().join("blk-index");

  println!("Refreshing block index copy...");
  println!("  Source: {}", live_index.display());
  println!("  Dest:   {}", copy_dir.display());

  let (copied, skipped) = refresh_index_copy(&live_index, &copy_dir)?;

  println!("Done: {copied} files updated, {skipped} files already up-to-date.");
  println!("dog index update will now use direct .blk file reads.");

  Ok(None)
}
