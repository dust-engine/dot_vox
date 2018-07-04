use nom::{le_f32, le_u32};

/// The type of a material
#[derive(Debug, PartialEq)]
pub enum MaterialType {
    /// A diffuse material.
    Diffuse,
    /// A metallic material, float indicating the blend between metal and diffuse material.
    Metal(f32),
    /// A glass material, float indicating the blend between glass and diffuse material.
    Glass(f32),
    /// An emissive material, float indicating the degree of self-illumination.
    Emissive(f32),
    /// An unknown material type.
    Unknown(u32, f32),
}

impl MaterialType {
    /// Instantiates a MaterialType from an identifier and a weight value.
    pub fn from_u32(material_type: u32, weight: f32) -> MaterialType {
        match material_type {
            0 => MaterialType::Diffuse,
            1 => MaterialType::Metal(weight),
            2 => MaterialType::Glass(weight),
            3 => MaterialType::Emissive(weight),
            _ => MaterialType::Unknown(material_type, weight),
        }
    }
}

named!(pub parse_material_type <&[u8], MaterialType>, do_parse!(
    material_type: call!(le_u32) >>
    weight: call!(le_f32) >>
    (MaterialType::from_u32(material_type, weight))
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_diffuse_material_type() {
        let bytes = include_bytes!("../resources/valid_diffuse_material_type.bytes").to_vec();
        let result = super::parse_material_type(&bytes);
        assert!(result.is_ok());
        let (_, material_type) = result.unwrap();
        assert_eq!(MaterialType::Diffuse, material_type);
    }

    #[test]
    fn can_parse_metal_material_type() {
        let bytes = include_bytes!("../resources/valid_material_type.bytes").to_vec();
        let result = super::parse_material_type(&bytes);
        assert!(result.is_ok());
        let (_, material_type) = result.unwrap();
        assert_eq!(MaterialType::Metal(1.0f32), material_type);
    }
}
