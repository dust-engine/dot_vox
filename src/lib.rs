//! Load MagicaVoxel .vox files into Rust
#![deny(missing_docs)]

#[macro_use]
extern crate bitflags;
extern crate byteorder;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate nom;

#[cfg(test)]
extern crate avow;

mod dot_vox_data;
mod pallete;
mod material;
mod model;

pub use dot_vox_data::DotVoxData;

pub use material::material::Material;
pub use material::material_properties::MaterialProperties;
pub use material::material_type::MaterialType;

pub use model::model::Model;
pub use model::size::Size;
pub use model::voxel::Voxel;

pub use pallete::DEFAULT_PALLETE;

use material::material::extract_materials;
use model::model::extract_models;
use pallete::extract_pallete;

use nom::IResult::Done;
use nom::le_u32;

use std::fs::File;
use std::io::Read;

const MAGIC_NUMBER: &'static str = "VOX ";

named!(parse_vox_file <&[u8], DotVoxData>, do_parse!(
  tag!(MAGIC_NUMBER) >>
  version: le_u32  >>
  take!(12)          >>
  models: extract_models >>
  pallete: opt_res!(extract_pallete) >>
  opt_res!(complete!(take!(4))) >>
  materials: opt_res!(extract_materials) >>
  (DotVoxData {
    version: version, 
    models: models, 
    pallete: pallete.unwrap_or(DEFAULT_PALLETE.to_vec()),
    materials: materials.unwrap_or(vec![]),
  })
));

/// Loads the supplied MagicaVoxel .vox file
///
/// Loads the supplied file, parses it, and returns a `DotVoxData` containing 
/// the version of the MagicaVoxel file, a `Vec<Model>` containing all `Model`s 
/// contained within the file, a `Vec<u32>` containing the pallete information 
/// (RGBA), and a `Vec<Material>` containing all the specialized materials.
///
/// # Panics
/// No panics should occur with this library - if you find one, please raise a 
/// GitHub issue for it.
///
/// # Errors
/// All errors are strings, and should describe the issue that caused them to 
/// occur.
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
///   pallete: DEFAULT_PALLETE.to_vec(),
///   materials: vec![],
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
  use byteorder::{ByteOrder, LittleEndian};

    lazy_static! {
    /// The default pallete used by MagicaVoxel - this is supplied if no pallete is included in the .vox file.
    static ref MODIFIED_PALLETE: Vec<u32> = include_bytes!("resources/modified_pallete.bytes")
      .chunks(4)
      .map(LittleEndian::read_u32)
      .collect();
  }

    fn placeholder(pallete: Vec<u32>, materials: Vec<Material>) -> DotVoxData {
        DotVoxData {
            version: 150,
            models: vec![
                Model {
                    size: Size { x: 2, y: 2, z: 2 },
                    voxels: vec![
                        Voxel::new(0, 0, 0, 226),
                        Voxel::new(0, 1, 1, 216),
                        Voxel::new(1, 0, 1, 236),
                        Voxel::new(1, 1, 0, 6),
                    ],
                },
            ],
            pallete: pallete,
            materials: materials
        }
    }

    fn compare_data(actual: DotVoxData, expected: DotVoxData) {
        assert_eq!(actual.version, expected.version);
        assert_eq!(actual.models, expected.models);
        vec::are_eq(actual.pallete, expected.pallete);
        vec::are_eq(actual.materials, expected.materials);
    }

    #[test]
    fn valid_file_with_no_pallete_is_read_successfully() {
        let result = load("src/resources/placeholder.vox");
        assert!(result.is_ok());
        compare_data(result.unwrap(), placeholder(DEFAULT_PALLETE.to_vec(), vec![]));
    }

    #[test]
    fn valid_file_with_pallete_is_read_successfully() {
        let result = load("src/resources/placeholder-with-pallete.vox");
        assert!(result.is_ok());
        compare_data(result.unwrap(), placeholder(MODIFIED_PALLETE.to_vec(), vec![]));
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
        compare_data(models, placeholder(DEFAULT_PALLETE.to_vec(), vec![]));
    }

    #[test]
    fn can_parse_vox_file_with_pallete() {
        let bytes = include_bytes!("resources/placeholder-with-pallete.vox").to_vec();
        let result = super::parse_vox_file(&bytes);
        assert!(result.is_done());
        let (_, models) = result.unwrap();
        compare_data(models, placeholder(MODIFIED_PALLETE.to_vec(), vec![]));
    }

    #[test]
    fn can_parse_vox_file_with_pallete_and_materials() {
        let bytes = include_bytes!("resources/placeholder-with-materials.vox").to_vec();
        let result = super::parse_vox_file(&bytes);
        assert!(result.is_done());
        let (_, voxel_data) = result.unwrap();
        compare_data(voxel_data, placeholder(DEFAULT_PALLETE.to_vec(), vec![Material {
            id: 216,
            material_type: MaterialType::Metal(0.694737),
            properties: MaterialProperties {
                plastic: Some(1.0),
                roughness: Some(0.389474),
                specular: Some(0.821053),
                ior: Some(0.3),
                ..Default::default()
            },
        }]));
    }
}
