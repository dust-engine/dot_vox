use {MaterialProperties, MaterialType};
use material::material_type::parse_material_type;
use material::material_properties::parse_material_properties;
use nom::le_u32;

/// A material used to render this model.
#[derive(Debug, PartialEq)]
pub struct Material {
    /// The ID for this material
    pub id: u8,
    /// The type of material this is
    pub material_type: MaterialType,
    /// Additional property values.
    pub properties: MaterialProperties,
}

named!(parse_material <&[u8], Material>, do_parse!(
    take!(12)    >>
    id: le_u32 >>
    material_type: parse_material_type >>
    properties: parse_material_properties >>
    (Material {
      id: id as u8,
      material_type: material_type,
      properties: properties
    })
));

named!(pub extract_materials <&[u8], Vec<Material> >, do_parse!(
  models: many0!(complete!(parse_material)) >>
  (models)
));

#[cfg(test)]
mod tests {
    use super::*;
    use avow::vec;

    #[test]
    fn can_parse_material_chunk() {
        let bytes = include_bytes!("../resources/valid_material.bytes").to_vec();
        let result = super::parse_material(&bytes);
        assert!(result.is_done());
        let (_, material) = result.unwrap();
        assert_eq!(249, material.id);
        assert_eq!(MaterialType::Metal(1.0), material.material_type);
        assert_eq!(
            MaterialProperties {
                plastic: Some(1.0),
                roughness: Some(0.0),
                specular: Some(1.0),
                ior: Some(0.3),
                power: Some(4.0),
                glow: Some(0.589474),
                ..Default::default()
            },
            material.properties
        );
    }

    #[test]
    fn can_parse_multiple_materials() {
        let bytes = include_bytes!("../resources/multi-materials.bytes").to_vec();
        let result = super::extract_materials(&bytes);
        assert!(result.is_done());
        let (_, materials) = result.unwrap();
        vec::are_eq(
            materials,
            vec![
                Material {
                    id: 79,
                    material_type: MaterialType::Metal(1.0),
                    properties: MaterialProperties {
                        plastic: Some(0.0),
                        roughness: Some(0.1),
                        specular: Some(0.5),
                        ior: Some(0.3),
                        ..Default::default()
                    },
                },
                Material {
                    id: 85,
                    material_type: MaterialType::Metal(0.526316),
                    properties: MaterialProperties {
                        plastic: Some(0.0),
                        roughness: Some(0.252632),
                        specular: Some(0.736842),
                        ior: Some(0.3),
                        ..Default::default()
                    },
                },
                Material {
                    id: 249,
                    material_type: MaterialType::Glass(0.810526),
                    properties: MaterialProperties {
                        plastic: Some(0.0),
                        roughness: Some(0.189474),
                        specular: Some(0.5),
                        ior: Some(0.547368),
                        attenuation: Some(0.021053),
                        ..Default::default()
                    },
                },
            ],
        );
    }
}
