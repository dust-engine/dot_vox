use byteorder::{ByteOrder, LittleEndian};
use nom::types::CompleteByteSlice;
use crate::parser::le_u32;

lazy_static! {
  /// The default palette used by MagicaVoxel - this is supplied if no palette
  /// is included in the .vox file.
  pub static ref DEFAULT_PALETTE: Vec<u32> =
    include_bytes!("resources/default_palette.bytes")
        .chunks(4)
        .map(LittleEndian::read_u32)
        .collect();
}

named!(pub extract_palette <CompleteByteSlice, Vec<u32> >, do_parse!(
    res: many_till!(le_u32, eof!()) >>
    (res.0)
));
