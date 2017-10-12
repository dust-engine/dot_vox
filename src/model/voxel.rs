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
    pub i: u8,
}