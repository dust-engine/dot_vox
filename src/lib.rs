//! Load MagicaVoxel .vox files into Rust
#![deny(missing_docs)]

extern crate byteorder;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate nom;

#[cfg(test)]
extern crate avow;

mod dot_vox_data;
mod model;

pub use dot_vox_data::DotVoxData;
pub use model::model::Model;
pub use model::size::Size;
pub use model::voxel::Voxel;

use std::fs::File;
use std::io::Read;

use byteorder::{ByteOrder, LittleEndian};
use nom::IResult::Done;

const MAGIC_NUMBER: &'static str = "VOX ";

lazy_static! {
  /// The default pallete used by MagicaVoxel - this is supplied if no pallete is included in the .vox file.
  pub static ref DEFAULT_PALLETE: Vec<u32> = include_bytes!("resources/default_pallete.bytes")
    .chunks(4)
    .map(LittleEndian::read_u32)
    .collect();
}

named!(take_u32 <&[u8], u32>, map!(take!(4), LittleEndian::read_u32));
named!(take_u8 <&[u8], u8>, map!(take!(1), |u: &[u8]| *u.first().unwrap()));

named!(parse_voxel <&[u8], Voxel>, do_parse!(
  x: take_u8 >>
  y: take_u8 >>
  z: take_u8 >>
  i: take_u8 >>
  (Voxel { x: x, y: y, z: z, i: i })
));

named!(parse_voxels <&[u8], Vec<Voxel> >, do_parse!(
  take!(12)            >>
  num_voxels: take_u32 >>
  voxels: many_m_n!(num_voxels as usize, num_voxels as usize, parse_voxel) >>
  (voxels)
));

named!(parse_size <&[u8], Size>, do_parse!(
  take!(12)   >>
  x: take_u32 >>
  y: take_u32 >>
  z: take_u32 >>
  (Size { x: x, y: y, z: z })
));

named!(parse_model <&[u8], Model>, do_parse!(
  size: parse_size     >>
  voxels: parse_voxels >>
  (Model { size: size, voxels: voxels })
));

named!(parse_models <&[u8], Vec<Model> >, do_parse!(
  take!(12)             >>
  model_count: take_u32 >>
  models: many_m_n!(model_count as usize, model_count as usize, parse_model) >>
  (models)
));

named!(extract_models <&[u8], Vec<Model> >, switch!(peek!(take!(4)),
    b"PACK" => call!(parse_models) |
    b"SIZE" => map!(call!(parse_model), |m| vec!(m))
));

named!(parse_pallete <&[u8], Vec<u32> >, complete!(do_parse!(
    take!(8) >>
    colors: many_m_n!(256, 256, take_u32) >>
    (colors)
)));

named!(extract_pallete <&[u8], Vec<u32> >, complete!(switch!(peek!(take!(4)),
    b"RGBA" => call!(parse_pallete)
)));

named!(parse_vox_file <&[u8], DotVoxData>, do_parse!(
  tag!(MAGIC_NUMBER) >>
  version: take_u32  >>
  take!(12)          >>
  models: extract_models >>
  pallete: opt_res!(extract_pallete) >>
  (DotVoxData {
    version: version, 
    models: models, 
    pallete: pallete.unwrap_or(DEFAULT_PALLETE.to_vec())
  })
));

/// Loads the supplied MagicaVoxel .vox file
///
/// Loads the supplied file, parses it, and returns a `DotVoxData` containing the version of the
/// MagicaVoxel file, a `Vec<Model>` containing all `Model`s contained within the file, and a
/// `Vec<u32>` containing the pallete information (RGBA).
///
/// # Panics
/// No panics should occur with this library - if you find one, please raise a GitHub issue for it.
///
/// # Errors
/// All errors are strings, and should describe the issue that caused them to occur.
///
/// # Examples
///
/// Loading a file:
///
/// ```
/// use dot_vox::*;
///
/// let result = load("src/resources/placeholder.vox");
/// assert_eq!(result.unwrap(), DotVoxData {
///   version: 150,
///   models: vec!(
///     Model {
///       size: Size { x: 2, y: 2, z: 2 },
///       voxels: vec!(
///         Voxel { x: 0, y: 0, z: 0, i: 226 },
///         Voxel { x: 0, y: 1, z: 1, i: 216 },
///         Voxel { x: 1, y: 0, z: 1, i: 236 },
///         Voxel { x: 1, y: 1, z: 0, i: 6 }
///       )
///     }
///   ),
///   pallete: DEFAULT_PALLETE.to_vec()
/// });
/// ```
pub fn load(filename: &str) -> Result<DotVoxData, &'static str> {
  match File::open(filename) {
    Ok(mut f) => {
      let mut buffer = Vec::new();
      f.read_to_end(&mut buffer).expect("Unable to read file");
      match parse_vox_file(&buffer) {
        Done(_, parsed) => Ok(parsed),
        _ => Err("Not a valid MagicaVoxel .vox file"),
      }
    }
    Err(_) => Err("Unable to load file"),
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use avow::vec;

  lazy_static! {
    /// The default pallete used by MagicaVoxel - this is supplied if no pallete is included in the .vox file.
    static ref MODIFIED_PALLETE: Vec<u32> = include_bytes!("resources/modified_pallete.bytes")
      .chunks(4)
      .map(LittleEndian::read_u32)
      .collect();
  }
  
  fn placeholder(pallete: Vec<u32>) -> DotVoxData {
    DotVoxData {
      version: 150,
      models: vec![
        Model {
          size: Size { x: 2, y: 2, z: 2 },
          voxels: vec![
            Voxel {
              x: 0,
              y: 0,
              z: 0,
              i: 226,
            },
            Voxel {
              x: 0,
              y: 1,
              z: 1,
              i: 216,
            },
            Voxel {
              x: 1,
              y: 0,
              z: 1,
              i: 236,
            },
            Voxel {
              x: 1,
              y: 1,
              z: 0,
              i: 6,
            },
          ],
        },
      ],
      pallete: pallete,
    }
  }

  fn compare_data(actual: DotVoxData, expected: DotVoxData) {
    assert_eq!(actual.version, expected.version);
    assert_eq!(actual.models, expected.models);
    vec::are_eq(actual.pallete, expected.pallete);
  }

  #[test]
  fn valid_file_with_no_pallete_is_read_successfully() {
    let result = load("src/resources/placeholder.vox");
    assert!(result.is_ok());
    compare_data(result.unwrap(), placeholder(DEFAULT_PALLETE.to_vec()));
  }

  #[test]
  fn valid_file_with_pallete_is_read_successfully() {
    let result = load("src/resources/placeholder-with-pallete.vox");
    assert!(result.is_ok());
    compare_data(result.unwrap(), placeholder(MODIFIED_PALLETE.to_vec()));
  }

  #[test]
  fn not_present_file_causes_error() {
    let result = load("src/resources/not_here.vox");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Unable to load file");
  }

  #[test]
  fn non_vox_file_causes_error() {
    let result = load("src/resources/not_a.vox");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Not a valid MagicaVoxel .vox file");
  }

  #[test]
  fn can_parse_vox_file_without_pallete() {
    let bytes = include_bytes!("resources/placeholder.vox").to_vec();
    let result = super::parse_vox_file(&bytes);
    assert!(result.is_done());
    let (_, models) = result.unwrap();
    compare_data(models, placeholder(DEFAULT_PALLETE.clone()));
  }

  #[test]
  fn can_parse_vox_file_with_pallete() {
    let bytes = include_bytes!("resources/placeholder-with-pallete.vox").to_vec();
    let result = super::parse_vox_file(&bytes);
    assert!(result.is_done());
    let (_, models) = result.unwrap();
    compare_data(models, placeholder(MODIFIED_PALLETE.to_vec()));
  }

  #[test]
  fn can_parse_size_chunk() {
    let bytes = include_bytes!("resources/valid_size.bytes").to_vec();
    let result = super::parse_size(&bytes);
    assert!(result.is_done());
    let (_, size) = result.unwrap();
    assert_eq!(
      size,
      Size {
        x: 24,
        y: 24,
        z: 24,
      }
    );
  }

  #[test]
  fn can_parse_voxels_chunk() {
    let bytes = include_bytes!("resources/valid_voxels.bytes").to_vec();
    let result = super::parse_voxels(&bytes);
    assert!(result.is_done());
    let (_, voxels) = result.unwrap();
    vec::are_eq(
      voxels,
      vec![
        Voxel {
          x: 0,
          y: 12,
          z: 22,
          i: 226,
        },
        Voxel {
          x: 12,
          y: 23,
          z: 13,
          i: 226,
        },
      ],
    );
  }

  #[test]
  fn can_parse_pallete_chunk() {
    let bytes = include_bytes!("resources/valid_pallete.bytes").to_vec();
    let result = super::parse_pallete(&bytes);
    assert!(result.is_done());
    let (_, pallete) = result.unwrap();
    vec::are_eq(pallete, DEFAULT_PALLETE.to_vec());
  }
}
