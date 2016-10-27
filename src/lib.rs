#[macro_use]
extern crate nom;
extern crate byteorder;

use std::fs::File;
use std::io::Read;

use byteorder::{ByteOrder, LittleEndian};
use nom::IResult::Done;

const MAGIC_NUMBER: &'static str = "VOX ";

#[derive(Debug, PartialEq)]
pub struct DotVoxData {
    version: u32,
    models: Vec<Model>
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Voxel {
  x: u8,
  y: u8,
  z: u8,
  i: u8
}

#[derive(Debug, PartialEq)]
pub struct Size {
  x: u32,
  y: u32,
  z: u32
}

#[derive(Debug, PartialEq)]
pub struct Model {
  size: Size,
  voxels: Vec<Voxel>
}

named!(take_u32 <&[u8], u32>, map!(take!(4), LittleEndian::read_u32));
named!(take_u8 <&[u8], u8>, map!(take!(1), |u: &[u8]| *u.first().unwrap()));

named!(parse_voxel <&[u8], Voxel>, chain!(
  x: take_u8 ~
  y: take_u8 ~
  z: take_u8 ~
  i: take_u8,
  || Voxel { x: x, y: y, z: z, i: i }
));

named!(pub parse_voxels <&[u8], Vec<Voxel> >, chain!(
  take!(12) ~
  num_voxels: take_u32 ~
  voxels: many_m_n!(num_voxels as usize, num_voxels as usize, parse_voxel),
  || voxels
));

named!(pub parse_size <&[u8], Size>, chain!(
  take!(12) ~
  x: take_u32 ~
  y: take_u32 ~
  z: take_u32,
  || Size { x: x, y: y, z: z }
));

// named!(pub parse_main <&[u8], Vec<Model> >, chain!(
//   main_size: take_u32 ~
//   take!(4) ~
//   take!(main_size) ~
//   children: parse_chunks,
//   || children)
// );

named!(parse_model <&[u8], Model>, chain!(
  size: parse_size ~
  voxels: parse_voxels,
  || Model { size: size, voxels: voxels }
));

named!(parse_models <&[u8], Vec<Model> >, chain!(
  take!(12) ~
  model_count: take_u32 ~
  models: many_m_n!(model_count as usize, model_count as usize, parse_model),
  || models
));

named!(pub parse_vox_file <&[u8], DotVoxData>, chain!(
  tag!(MAGIC_NUMBER) ~
  version: take_u32 ~
  take!(12) ~
  models: switch!(peek!(take!(4)),
    b"PACK" => call!(parse_models) |
    b"SIZE" => map!(call!(parse_model), |m| vec!(m))
  ),
  || DotVoxData {
    version: version,
    models: models
  }
));

pub fn load(filename: &str) -> Result<DotVoxData, &'static str> {
  match File::open(filename) {
    Ok(mut f) => {
      let mut buffer = Vec::new();
      f.read_to_end(&mut buffer).expect("Unable to read file");
      match parse_vox_file(&buffer) {
        Done(_, parsed) => Ok(parsed),
        _ => Err("Not a valid MagicaVoxel .vox file")
      }
    },
    Err(_) => Err("Unable to load file")
  }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placeholder() -> DotVoxData {
      DotVoxData {
        version: 150,
        models: vec!(
          Model {
            size: Size { x: 2, y: 2, z: 2 },
            voxels: vec!(
              Voxel { x: 0, y: 0, z: 0, i: 226 },
              Voxel { x: 0, y: 1, z: 1, i: 216 },
              Voxel { x: 1, y: 0, z: 1, i: 236 },
              Voxel { x: 1, y: 1, z: 0, i: 6 }
            )
          }
        )
      }
    }

    #[test]
    fn valid_file_is_read_successfully() {
        let result = load("src/resources/placeholder.vox");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), placeholder());
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
    fn can_parse_vox_file() {
      let bytes = include_bytes!("resources/placeholder.vox").to_vec();
      let result = parse_vox_file(&bytes);
      assert!(result.is_done());
      let (_, models) = result.unwrap();
      assert_eq!(models, placeholder());
    }

    #[test]
    fn can_parse_size_chunk() {
      let bytes = include_bytes!("resources/valid_size.bytes").to_vec();
      let result = parse_size(&bytes);
      assert!(result.is_done());
      let (_, size) = result.unwrap();
      assert_eq!(size, Size { x: 24, y: 24, z: 24 } );
    }

    #[test]
    fn can_parse_voxels_chunk() {
      let bytes = include_bytes!("resources/valid_voxels.bytes").to_vec();
      let result = parse_voxels(&bytes);
      assert!(result.is_done());
      let (_, voxels) = result.unwrap();
      assert_eq!(voxels, vec!(Voxel { x: 0, y: 12, z: 22, i: 226 }, Voxel { x: 12, y: 23, z: 13, i: 226 }));
    }
}
