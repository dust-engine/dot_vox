use nom::{
    multi::count,
    number::complete::{le_u32, le_u8},
    sequence::tuple,
    IResult,
};

/// A renderable voxel model.
#[derive(Debug, PartialEq, Eq)]
pub struct Model {
    /// The size of the model in voxels.
    pub size: Size,
    /// The voxels to be displayed.
    pub voxels: Vec<Voxel>,
}

impl Model {
    /// Number of bytes when encoded in `.vox` format.
    pub fn num_vox_bytes(&self) -> u32 {
        // The number 40 comes from:
        // - 24 bytes for the chunk header format (SIZE/XYZI labels, chunk and child sizes, etc.)
        // - 12 bytes for the SIZE contents (x, y, z)
        // - 4 bytes for the voxel length u32
        40 + 4 * self.voxels.len() as u32
    }
}

/// The dimensions of a model in voxels.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Size {
    /// The width of the model in voxels.
    pub x: u32,
    /// The height of the model in voxels.
    pub y: u32,
    /// The depth of the model in voxels.
    pub z: u32,
}

/// A voxel.
///
/// A point in 3D space, with an indexed color attached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Voxel {
    /// The X coordinate for the voxel.
    pub x: u8,
    /// The Y coordinate for the voxel.
    pub y: u8,
    /// The Z coordinate for the voxel.
    pub z: u8,
    /// Index in the color palette. Note that this will be oen less than the
    /// value stored in the source file, as the palette indices run from
    /// 1--255, whereas in memory the indices run from 0--254. Therefore, to
    /// make life easier, we store the in-memory index here. Should you
    /// require the source file's indices, simply add 1 to this value.
    pub i: u8,
}

pub fn parse_size(i: &[u8]) -> IResult<&[u8], Size> {
    let (i, (x, y, z)) = tuple((le_u32, le_u32, le_u32))(i)?;
    Ok((i, Size { x, y, z }))
}

fn parse_voxel(input: &[u8]) -> IResult<&[u8], Voxel> {
    let (input, (x, y, z, i)) = tuple((le_u8, le_u8, le_u8, le_u8))(input)?;
    Ok((
        input,
        Voxel {
            x,
            y,
            z,
            i: i.saturating_sub(1),
        },
    ))
}

pub fn parse_voxels(i: &[u8]) -> IResult<&[u8], Vec<Voxel>> {
    let (i, n) = le_u32(i)?;
    count(parse_voxel, n as usize)(i)
}
