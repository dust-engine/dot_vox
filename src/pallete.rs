use take_u32;
use byteorder::{ByteOrder, LittleEndian};

lazy_static! {
  /// The default pallete used by MagicaVoxel - this is supplied if no pallete
  /// is included in the .vox file.
  pub static ref DEFAULT_PALLETE: Vec<u32> =
    include_bytes!("resources/default_pallete.bytes")
        .chunks(4)
        .map(LittleEndian::read_u32)
        .collect();
}

named!(parse_pallete <&[u8], Vec<u32> >, complete!(do_parse!(
    take!(8) >>
    colors: many_m_n!(256, 256, take_u32) >>
    (colors)
)));

named!(pub extract_pallete <&[u8], Vec<u32> >, complete!(switch!(peek!(take!(4)),
    b"RGBA" => call!(parse_pallete)
)));

#[cfg(test)]
mod tests {
    use super::*;
    use avow::vec;

    #[test]
    fn can_parse_pallete_chunk() {
        let bytes = include_bytes!("resources/valid_pallete.bytes").to_vec();
        let result = super::parse_pallete(&bytes);
        assert!(result.is_done());
        let (_, pallete) = result.unwrap();
        vec::are_eq(pallete, DEFAULT_PALLETE.to_vec());
    }
}
