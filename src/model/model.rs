use Size;
use Voxel;

/// A renderable voxel Model
#[derive(Debug, PartialEq)]
pub struct Model {
    /// The size of the model in voxels
    pub size: Size,
    /// The voxels to be displayed.
    pub voxels: Vec<Voxel>,
}
