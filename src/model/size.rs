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
    pub z: u32,
}