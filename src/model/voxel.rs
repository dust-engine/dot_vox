use nom::{le_u8, le_u32};

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

impl Voxel {
    /// Instantiate a Voxel.
    pub fn new(x: u8, y: u8, z: u8, i: u8) -> Voxel {
        Voxel {
            x: x,
            y: y,
            z: z,
            i: i,
        }
    }
}

named!(parse_voxel <&[u8], Voxel>, do_parse!(
  x: le_u8 >>
  y: le_u8 >>
  z: le_u8 >>
  i: le_u8 >>
  (Voxel::new(x, y, z, i.saturating_sub(1)))
));

named!(pub parse_voxels <&[u8], Vec<Voxel> >, do_parse!(
  take!(12)            >>
  num_voxels: le_u32 >>
  voxels: many_m_n!(num_voxels as usize, num_voxels as usize, parse_voxel) >>
  (voxels)
));

#[cfg(test)]
mod tests {
    use super::*;
    use avow::vec;

    #[test]
    fn can_parse_voxels_chunk() {
        let bytes = include_bytes!("../resources/valid_voxels.bytes").to_vec();
        let result = super::parse_voxels(&bytes);
        assert!(result.is_done());
        let (_, voxels) = result.unwrap();
        vec::are_eq(
            voxels,
            vec![Voxel::new(0, 12, 22, 225), Voxel::new(12, 23, 13, 225)],
        );
    }
}
