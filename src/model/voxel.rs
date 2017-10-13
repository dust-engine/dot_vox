use {take_u32, take_u8};

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
  x: take_u8 >>
  y: take_u8 >>
  z: take_u8 >>
  i: take_u8 >>
  (Voxel::new(x, y, z, i))
));

named!(pub parse_voxels <&[u8], Vec<Voxel> >, do_parse!(
  take!(12)            >>
  num_voxels: take_u32 >>
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
            vec![Voxel::new(0, 12, 22, 226), Voxel::new(12, 23, 13, 226)],
        );
    }
}
