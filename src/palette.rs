use byteorder::{ByteOrder, LittleEndian};
use nom::combinator::all_consuming;
use nom::multi::many0;
use nom::number::complete::le_u32;
use nom::IResult;

lazy_static! {
  /// The default palette used by MagicaVoxel - this is supplied if no palette
  /// is included in the .vox file.
  pub static ref DEFAULT_PALETTE: Vec<u32> =
    include_bytes!("resources/default_palette.bytes")
        .chunks(4)
        .map(LittleEndian::read_u32)
        .collect();
}

pub fn extract_palette(i: &[u8]) -> IResult<&[u8], Vec<u32>> {
    all_consuming(many0(le_u32))(i)
}
