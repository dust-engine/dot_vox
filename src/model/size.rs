use nom::le_u32;

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

named!(pub parse_size <&[u8], Size>, do_parse!(
  x: le_u32 >>
  y: le_u32 >>
  z: le_u32 >>
  (Size { x: x, y: y, z: z })
));


#[cfg(test)]
mod tests {
    use super::*;


}
