use byteorder::{ByteOrder, LittleEndian};
use nom::le_u32;

lazy_static! {
  /// The default palette used by MagicaVoxel - this is supplied if no palette
  /// is included in the .vox file.
  pub static ref DEFAULT_PALETTE: Vec<u32> =
    include_bytes!("resources/default_palette.bytes")
        .chunks(4)
        .map(LittleEndian::read_u32)
        .collect();
}

named!(pub extract_palette <&[u8], Vec<u32> >, complete!(do_parse!(
    take!(12) >>
    colors: many_m_n!(255, 255, le_u32) >>
    (colors)
)));

#[cfg(test)]
mod tests {
    use super::*;
    use avow::vec;

    #[test]
    fn can_parse_palette_chunk() {
        let bytes = include_bytes!("resources/valid_palette.bytes").to_vec();
        let result = super::extract_palette(&bytes);
        assert!(result.is_done());
        let (_, palette) = result.unwrap();
        vec::are_eq(palette, DEFAULT_PALETTE.to_vec());
    }
}
