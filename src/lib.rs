//! Load MagicaVoxel .vox files into Rust
#![deny(missing_docs)]

#[macro_use]
extern crate nom;
extern crate byteorder;

use std::fs::File;
use std::io::Read;

use byteorder::{ByteOrder, LittleEndian};
use nom::IResult::Done;

const MAGIC_NUMBER: &'static str = "VOX ";

/// The default indexed color pallete used by MagicaVoxel files.
pub const DEFAULT_PALLETE: [u32; 256] = [
	0x00000000, 0xffffffff, 0xffccffff, 0xff99ffff, 0xff66ffff, 0xff33ffff, 0xff00ffff, 0xffffccff, 0xffccccff, 0xff99ccff, 0xff66ccff, 0xff33ccff, 0xff00ccff, 0xffff99ff, 0xffcc99ff, 0xff9999ff,
	0xff6699ff, 0xff3399ff, 0xff0099ff, 0xffff66ff, 0xffcc66ff, 0xff9966ff, 0xff6666ff, 0xff3366ff, 0xff0066ff, 0xffff33ff, 0xffcc33ff, 0xff9933ff, 0xff6633ff, 0xff3333ff, 0xff0033ff, 0xffff00ff,
	0xffcc00ff, 0xff9900ff, 0xff6600ff, 0xff3300ff, 0xff0000ff, 0xffffffcc, 0xffccffcc, 0xff99ffcc, 0xff66ffcc, 0xff33ffcc, 0xff00ffcc, 0xffffcccc, 0xffcccccc, 0xff99cccc, 0xff66cccc, 0xff33cccc,
	0xff00cccc, 0xffff99cc, 0xffcc99cc, 0xff9999cc, 0xff6699cc, 0xff3399cc, 0xff0099cc, 0xffff66cc, 0xffcc66cc, 0xff9966cc, 0xff6666cc, 0xff3366cc, 0xff0066cc, 0xffff33cc, 0xffcc33cc, 0xff9933cc,
	0xff6633cc, 0xff3333cc, 0xff0033cc, 0xffff00cc, 0xffcc00cc, 0xff9900cc, 0xff6600cc, 0xff3300cc, 0xff0000cc, 0xffffff99, 0xffccff99, 0xff99ff99, 0xff66ff99, 0xff33ff99, 0xff00ff99, 0xffffcc99,
	0xffcccc99, 0xff99cc99, 0xff66cc99, 0xff33cc99, 0xff00cc99, 0xffff9999, 0xffcc9999, 0xff999999, 0xff669999, 0xff339999, 0xff009999, 0xffff6699, 0xffcc6699, 0xff996699, 0xff666699, 0xff336699,
	0xff006699, 0xffff3399, 0xffcc3399, 0xff993399, 0xff663399, 0xff333399, 0xff003399, 0xffff0099, 0xffcc0099, 0xff990099, 0xff660099, 0xff330099, 0xff000099, 0xffffff66, 0xffccff66, 0xff99ff66,
	0xff66ff66, 0xff33ff66, 0xff00ff66, 0xffffcc66, 0xffcccc66, 0xff99cc66, 0xff66cc66, 0xff33cc66, 0xff00cc66, 0xffff9966, 0xffcc9966, 0xff999966, 0xff669966, 0xff339966, 0xff009966, 0xffff6666,
	0xffcc6666, 0xff996666, 0xff666666, 0xff336666, 0xff006666, 0xffff3366, 0xffcc3366, 0xff993366, 0xff663366, 0xff333366, 0xff003366, 0xffff0066, 0xffcc0066, 0xff990066, 0xff660066, 0xff330066,
	0xff000066, 0xffffff33, 0xffccff33, 0xff99ff33, 0xff66ff33, 0xff33ff33, 0xff00ff33, 0xffffcc33, 0xffcccc33, 0xff99cc33, 0xff66cc33, 0xff33cc33, 0xff00cc33, 0xffff9933, 0xffcc9933, 0xff999933,
	0xff669933, 0xff339933, 0xff009933, 0xffff6633, 0xffcc6633, 0xff996633, 0xff666633, 0xff336633, 0xff006633, 0xffff3333, 0xffcc3333, 0xff993333, 0xff663333, 0xff333333, 0xff003333, 0xffff0033,
	0xffcc0033, 0xff990033, 0xff660033, 0xff330033, 0xff000033, 0xffffff00, 0xffccff00, 0xff99ff00, 0xff66ff00, 0xff33ff00, 0xff00ff00, 0xffffcc00, 0xffcccc00, 0xff99cc00, 0xff66cc00, 0xff33cc00,
	0xff00cc00, 0xffff9900, 0xffcc9900, 0xff999900, 0xff669900, 0xff339900, 0xff009900, 0xffff6600, 0xffcc6600, 0xff996600, 0xff666600, 0xff336600, 0xff006600, 0xffff3300, 0xffcc3300, 0xff993300,
	0xff663300, 0xff333300, 0xff003300, 0xffff0000, 0xffcc0000, 0xff990000, 0xff660000, 0xff330000, 0xff0000ee, 0xff0000dd, 0xff0000bb, 0xff0000aa, 0xff000088, 0xff000077, 0xff000055, 0xff000044,
	0xff000022, 0xff000011, 0xff00ee00, 0xff00dd00, 0xff00bb00, 0xff00aa00, 0xff008800, 0xff007700, 0xff005500, 0xff004400, 0xff002200, 0xff001100, 0xffee0000, 0xffdd0000, 0xffbb0000, 0xffaa0000,
	0xff880000, 0xff770000, 0xff550000, 0xff440000, 0xff220000, 0xff110000, 0xffeeeeee, 0xffdddddd, 0xffbbbbbb, 0xffaaaaaa, 0xff888888, 0xff777777, 0xff555555, 0xff444444, 0xff222222, 0xff111111
];

/// Container for .vox file data
#[derive(Debug, PartialEq)]
pub struct DotVoxData {
    /// The version number of the .vox file.
    pub version: u32,
    /// A Vec of all the models contained within this file.
    pub models: Vec<Model>,
    /// A Vec containing the colour pallete as 32-bit integers
    pub pallete: Vec<u32>,
}

/// A Voxel
///
/// A Voxel is a point in 3D space, with an indexed colour attached.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Voxel {
  /// The X coordinate for the Voxel
  pub x: u8,
  /// The Y coordinate for the Voxel
  pub y: u8,
  /// The Z coordinate for the Voxel
  pub z: u8,
  /// Index in the Color Pallete
  pub i: u8
}

/// The size of a model in voxels
///
/// Indicates the size of the model in Voxels.
#[derive(Debug, PartialEq)]
pub struct Size {
  /// The width of the model in voxels.
  pub x: u32,
  /// The height of the model in voxels.
  pub y: u32,
  /// The depth of the model in voxels.
  pub z: u32
}

/// A renderable voxel Model
#[derive(Debug, PartialEq)]
pub struct Model {
  /// The size of the model in voxels
  pub size: Size,
  /// The voxels to be displayed.
  pub voxels: Vec<Voxel>
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

named!(parse_pallete <&[u8], Vec<u32> >, do_parse!(
    take!(8) >>
    colors: many_m_n!(256, 256, take_u32) >>
    (colors)
));

named!(parse_vox_file <&[u8], DotVoxData>, do_parse!(
  tag!(MAGIC_NUMBER) >>
  version: take_u32  >>
  take!(12)          >>
  models: switch!(peek!(take!(4)),
    b"PACK" => call!(parse_models) |
    b"SIZE" => map!(call!(parse_model), |m| vec!(m))
  ) >>
  pallete: switch!(take!(4),
    b"RGBA" => call!(parse_pallete) |
    _ => value!(Vec::<u32>::new())
  ) >>
  (DotVoxData {
    version: version,
    models: models,
    pallete: if pallete.len() == 0 {DEFAULT_PALLETE.to_vec()} else {pallete},
  })
));

/// Loads the supplied MagicaVoxel .vox file
///
/// Loads the supplied file, parses it, and returns a `DotVoxData` containing the version of the
/// MagicaVoxel file, and `Vec<Model>` containing all `Model`s contained within the file.
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
        ),
        pallete: DEFAULT_PALLETE.to_vec()
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
      let result = super::parse_vox_file(&bytes);
      assert!(result.is_done());
      let (_, models) = result.unwrap();
      assert_eq!(models, placeholder());
    }

    #[test]
    fn can_parse_size_chunk() {
      let bytes = include_bytes!("resources/valid_size.bytes").to_vec();
      let result = super::parse_size(&bytes);
      assert!(result.is_done());
      let (_, size) = result.unwrap();
      assert_eq!(size, Size { x: 24, y: 24, z: 24 } );
    }

    #[test]
    fn can_parse_voxels_chunk() {
      let bytes = include_bytes!("resources/valid_voxels.bytes").to_vec();
      let result = super::parse_voxels(&bytes);
      assert!(result.is_done());
      let (_, voxels) = result.unwrap();
      assert_eq!(voxels, vec!(Voxel { x: 0, y: 12, z: 22, i: 226 }, Voxel { x: 12, y: 23, z: 13, i: 226 }));
    }
}
