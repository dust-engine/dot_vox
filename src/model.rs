use nom::multi::count;
use nom::number::complete::{le_u32, le_u8};
use nom::sequence::tuple;
use nom::IResult;

/// A renderable voxel Model
#[derive(Debug, PartialEq)]
pub struct Model {
    /// The size of the model in voxels
    pub size: Size,
    /// The voxels to be displayed.
    pub voxels: Vec<Voxel>,
}

impl Model {
    /// Number of bytes when encoded in VOX format.
    pub fn num_vox_bytes(&self) -> u32 {
        12 + 4 * self.voxels.len() as u32
    }
}

/// The size of a model in voxels
///
/// Indicates the size of the model in Voxels.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Size {
    /// The width of the model in voxels.
    pub x: u32,
    /// The height of the model in voxels.
    pub y: u32,
    /// The depth of the model in voxels.
    pub z: u32,
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
    /// Index in the Color Palette. Note that this will be 1 less than the value stored in the
    /// source file, as the palette indices run from 1-255, whereas in memory the indices run from
    /// 0-254. Therefore, to make life easier, we store the in-memory index here. Should you require
    /// the source file's indices, simply add 1 to this value.
    pub i: u8,
}

pub fn parse_size(i: &[u8]) -> IResult<&[u8], Size> {
    let (i, (x, y, z)) = tuple((le_u32, le_u32, le_u32))(i)?;
    Ok((i, Size { x, y, z }))
}

fn parse_voxel(input: &[u8]) -> IResult<&[u8], Voxel> {
    let (input, (x, y, z, i)) = tuple((le_u8, le_u8, le_u8, le_u8))(input)?;
    Ok((input, Voxel { x, y, z, i: i.saturating_sub(1) }))
}

pub fn parse_voxels(i: &[u8]) -> IResult<&[u8], Vec<Voxel>> {
    let (i, n) = le_u32(i)?;
    count(parse_voxel, n as usize)(i)
}
