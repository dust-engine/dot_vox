//! Load MagicaVoxel .vox files into Rust
#![deny(missing_docs)]

extern crate byteorder;
#[cfg(test)]
extern crate env_logger;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;

#[cfg(test)]
extern crate avow;

mod dot_vox_data;
mod model;
mod palette;
mod parser;

pub use crate::dot_vox_data::DotVoxData;

pub use crate::parser::{Dict, Material};

pub use crate::model::Model;
pub use crate::model::Size;
pub use crate::model::Voxel;

use nom::types::CompleteByteSlice;

pub use crate::palette::DEFAULT_PALETTE;

use crate::parser::parse_vox_file;

use std::fs::File;
use std::io::Read;

/// Loads the supplied MagicaVoxel .vox file
///
/// Loads the supplied file, parses it, and returns a `DotVoxData` containing
/// the version of the MagicaVoxel file, a `Vec<Model>` containing all `Model`s
/// contained within the file, a `Vec<u32>` containing the palette information
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
///         Voxel { x: 0, y: 0, z: 0, i: 225 },
///         Voxel { x: 0, y: 1, z: 1, i: 215 },
///         Voxel { x: 1, y: 0, z: 1, i: 235 },
///         Voxel { x: 1, y: 1, z: 0, i: 5 }
///       )
///     }
///   ),
///   palette: DEFAULT_PALETTE.to_vec(),
///   materials: (0..256).into_iter()
///     .map(|i| Material {
///       id: i,
///       properties: {
///         let mut map = Dict::new();
///         map.insert("_ior".to_owned(), "0.3".to_owned());
///         map.insert("_spec".to_owned(), "0.5".to_owned());
///         map.insert("_rough".to_owned(), "0.1".to_owned());
///         map.insert("_type".to_owned(), "_diffuse".to_owned());
///         map.insert("_weight".to_owned(), "1".to_owned());
///         map
///       }
///     })
///     .collect(),
///   });
/// ```
pub fn load(filename: &str) -> Result<DotVoxData, &'static str> {
    match File::open(filename) {
        Ok(mut f) => {
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer).expect("Unable to read file");
            load_bytes(&buffer)
        }
        Err(_) => Err("Unable to load file"),
    }
}

/// Parses the byte array as a .vox file.
///
/// Parses the byte array and returns a `DotVoxData` containing  the version of the MagicaVoxel
/// file, a `Vec<Model>` containing all `Model`s contained within the file, a `Vec<u32>` containing
/// the palette information (RGBA), and a `Vec<Material>` containing all the specialized materials.
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
/// Reading a byte array:
///
/// ```
/// use dot_vox::*;
///
/// let result = load_bytes(include_bytes!("resources/placeholder.vox"));
/// assert_eq!(result.unwrap(), DotVoxData {
///   version: 150,
///   models: vec!(
///     Model {
///       size: Size { x: 2, y: 2, z: 2 },
///       voxels: vec!(
///         Voxel { x: 0, y: 0, z: 0, i: 225 },
///         Voxel { x: 0, y: 1, z: 1, i: 215 },
///         Voxel { x: 1, y: 0, z: 1, i: 235 },
///         Voxel { x: 1, y: 1, z: 0, i: 5 }
///       )
///     }
///   ),
///   palette: DEFAULT_PALETTE.to_vec(),
///   materials: (0..256).into_iter()
///     .map(|i| Material {
///       id: i,
///       properties: {
///         let mut map = Dict::new();
///         map.insert("_ior".to_owned(), "0.3".to_owned());
///         map.insert("_spec".to_owned(), "0.5".to_owned());
///         map.insert("_rough".to_owned(), "0.1".to_owned());
///         map.insert("_type".to_owned(), "_diffuse".to_owned());
///         map.insert("_weight".to_owned(), "1".to_owned());
///         map
///       }
///     })
///     .collect(),
///   });
/// ```
pub fn load_bytes(bytes: &[u8]) -> Result<DotVoxData, &'static str> {
    match parse_vox_file(CompleteByteSlice(bytes)) {
        Ok((_, parsed)) => Ok(parsed),
        Err(_) => Err("Not a valid MagicaVoxel .vox file"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use avow::vec;

    lazy_static! {
        static ref DEFAULT_MATERIALS: Vec<Material> = (0..256)
            .into_iter()
            .map(|i| Material {
                id: i,
                properties: {
                    let mut map = Dict::new();
                    map.insert("_ior".to_owned(), "0.3".to_owned());
                    map.insert("_spec".to_owned(), "0.5".to_owned());
                    map.insert("_rough".to_owned(), "0.1".to_owned());
                    map.insert("_type".to_owned(), "_diffuse".to_owned());
                    map.insert("_weight".to_owned(), "1".to_owned());
                    map
                }
            })
            .collect();
    }

    fn placeholder(palette: Vec<u32>, materials: Vec<Material>) -> DotVoxData {
        DotVoxData {
            version: 150,
            models: vec![Model {
                size: Size { x: 2, y: 2, z: 2 },
                voxels: vec![
                    Voxel {
                        x: 0,
                        y: 0,
                        z: 0,
                        i: 225,
                    },
                    Voxel {
                        x: 0,
                        y: 1,
                        z: 1,
                        i: 215,
                    },
                    Voxel {
                        x: 1,
                        y: 0,
                        z: 1,
                        i: 235,
                    },
                    Voxel {
                        x: 1,
                        y: 1,
                        z: 0,
                        i: 5,
                    },
                ],
            }],
            palette: palette,
            materials: materials,
        }
    }

    fn compare_data(actual: DotVoxData, expected: DotVoxData) {
        assert_eq!(actual.version, expected.version);
        assert_eq!(actual.models.len(), expected.models.len());
        actual
            .models
            .into_iter()
            .zip(expected.models.into_iter())
            .for_each(|(actual, expected)| {
                assert_eq!(actual.size, expected.size);
                vec::are_eq(actual.voxels, expected.voxels);
            });
        vec::are_eq(actual.palette, expected.palette);
        vec::are_eq(actual.materials, expected.materials);
    }

    #[test]
    fn valid_file_with_palette_is_read_successfully() {
        let result = load("src/resources/placeholder.vox");
        assert!(result.is_ok());
        compare_data(
            result.unwrap(),
            placeholder(DEFAULT_PALETTE.to_vec(), DEFAULT_MATERIALS.to_vec()),
        );
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
    fn can_parse_vox_file_with_palette() {
        let bytes = include_bytes!("resources/placeholder.vox").to_vec();
        let result = super::parse_vox_file(CompleteByteSlice(&bytes));
        assert!(result.is_ok());
        let (_, models) = result.unwrap();
        compare_data(
            models,
            placeholder(DEFAULT_PALETTE.to_vec(), DEFAULT_MATERIALS.to_vec()),
        );
    }

    #[test]
    fn can_parse_vox_file_with_materials() {
        let _log = env_logger::init();
        let bytes = include_bytes!("resources/placeholder-with-materials.vox").to_vec();
        let result = super::parse_vox_file(CompleteByteSlice(&bytes));
        assert!(result.is_ok());
        let (_, voxel_data) = result.unwrap();
        let mut materials: Vec<Material> = DEFAULT_MATERIALS.to_vec();
        materials[216] = Material {
            id: 216,
            properties: {
                let mut map = Dict::new();
                map.insert("_ior".to_owned(), "0.3".to_owned());
                map.insert("_spec".to_owned(), "0.821053".to_owned());
                map.insert("_rough".to_owned(), "0.389474".to_owned());
                map.insert("_type".to_owned(), "_metal".to_owned());
                map.insert("_plastic".to_owned(), "1".to_owned());
                map.insert("_weight".to_owned(), "0.694737".to_owned());
                map
            },
        };
        compare_data(voxel_data, placeholder(DEFAULT_PALETTE.to_vec(), materials));
    }

    fn write_and_load(data: DotVoxData) {
        let mut buffer = Vec::new();
        let write_result = data.write_vox(&mut buffer);
        assert!(write_result.is_ok());
        let load_result = load_bytes(&buffer);
        assert!(load_result.is_ok());
        compare_data(load_result.unwrap(), data);
    }

    #[test]
    fn can_write_vox_format_without_palette_nor_materials() {
        write_and_load(placeholder(Vec::new(), Vec::new()));
    }

    #[test]
    fn can_write_vox_format_without_materials() {
        write_and_load(placeholder(DEFAULT_PALETTE.to_vec(), Vec::new()));
    }
}
