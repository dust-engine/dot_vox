use nom::bytes::complete::take;
use nom::{IResult, Parser, combinator::all_consuming, multi::many0, number::complete::le_u8};

lazy_static! {
  /// The default palette used by [MagicaVoxel](https://ephtracy.github.io/) -- this is supplied if no palette
  /// is included in the `.vox` file.
  pub static ref DEFAULT_PALETTE: Vec<Color> =
    include_bytes!("resources/default_palette.bytes")
        .chunks(4)
        .map(|bytes| parse_color(bytes).unwrap().1)
        .collect();
}

/// The default index map, that sends [`crate::model::Voxel::i`] values to
/// corresponding palette slots.
pub const DEFAULT_INDEX_MAP: &[u8] = &create_default_index_map();

pub fn extract_index_map(i: &[u8]) -> IResult<&[u8], &[u8]> {
    all_consuming(take(256usize)).parse(i)
}

pub fn extract_palette(i: &[u8]) -> IResult<&[u8], Vec<Color>> {
    all_consuming(many0(parse_color)).parse(i)
}

fn parse_color(input: &[u8]) -> IResult<&[u8], Color> {
    let (input, (r, g, b, a)) = (le_u8, le_u8, le_u8, le_u8).parse(input)?;
    Ok((input, Color { r, g, b, a }))
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<Color> for [u8; 4] {
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}
impl From<&Color> for [u8; 4] {
    fn from(color: &Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

/// Creates an identity index map.
const fn create_default_index_map() -> [u8; 256] {
    let mut result = [0; 256];
    let mut i = 0;
    while i < result.len() {
        result[i] = i.wrapping_add(1) as u8;
        i += 1;
    }
    result
}
