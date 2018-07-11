use nom::{le_f32, le_u32};

/// A set of optional properties for the material.
#[derive(Debug, Default, PartialEq)]
pub struct MaterialProperties {
    /// If set, the degree of plasticisation.
    pub plastic: Option<f32>,
    /// If set, the degree of roughness.
    pub roughness: Option<f32>,
    /// If set, the degree of specular.
    pub specular: Option<f32>,
    /// Index Of Refraction
    pub ior: Option<f32>,
    /// Attenuation
    pub attenuation: Option<f32>,
    /// Power
    pub power: Option<f32>,
    /// Glow
    pub glow: Option<f32>,
    /// If set, whether isTotalPower.
    pub is_total_power: Option<bool>,
}

bitflags! {
    struct Properties: u32 {
        const PLASTIC         = 0x0000_0001;
        const ROUGHNESS       = 0x0000_0002;
        const SPECULAR        = 0x0000_0004;
        const IOR             = 0x0000_0008;
        const ATTENUATION     = 0x0000_0010;
        const POWER           = 0x0000_0020;
        const GLOW            = 0x0000_0040;
        const IS_TOTAL_POWER  = 0x0000_0080;
    }
}

fn properties_from_mask(bitmask: Properties, mut values: Vec<f32>) -> MaterialProperties {
    fn set_if_defined<T>(
        bitmask: Properties,
        property: Properties,
        values: &mut Vec<T>,
    ) -> Option<T> {
        if bitmask.contains(property) && !values.is_empty() {
            Some(values.remove(0))
        } else {
            None
        }
    }

    let mut material_properties = MaterialProperties::default();
    material_properties.plastic = set_if_defined(bitmask, Properties::PLASTIC, &mut values);
    material_properties.roughness = set_if_defined(bitmask, Properties::ROUGHNESS, &mut values);
    material_properties.specular = set_if_defined(bitmask, Properties::SPECULAR, &mut values);
    material_properties.ior = set_if_defined(bitmask, Properties::IOR, &mut values);
    material_properties.attenuation = set_if_defined(bitmask, Properties::ATTENUATION, &mut values);
    material_properties.power = set_if_defined(bitmask, Properties::POWER, &mut values);
    material_properties.glow = set_if_defined(bitmask, Properties::GLOW, &mut values);
    material_properties.is_total_power =
        set_if_defined(bitmask, Properties::IS_TOTAL_POWER, &mut vec![true]);
    material_properties
}

named!(pub parse_material_properties <&[u8], MaterialProperties>, do_parse!(
    bitmask: map!(call!(le_u32), Properties::from_bits_truncate) >>
    values: many_till!(call!(le_f32), alt_complete!(peek!(tag!("MATT")) | eof!())) >>
    (properties_from_mask(bitmask, values.0))
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_populate_properties_from_mask() {
        let properties = Properties::PLASTIC | Properties::IOR | Properties::IS_TOTAL_POWER;
        let values = vec![0.75, 0.31415];
        let material_properties = properties_from_mask(properties, values);
        assert_eq!(Some(0.75), material_properties.plastic);
        assert_eq!(Some(0.31415), material_properties.ior);
        assert_eq!(Some(true), material_properties.is_total_power);
    }

    #[test]
    fn properties_default_to_none_when_no_value_exists() {
        let properties = Properties::SPECULAR | Properties::ROUGHNESS | Properties::ATTENUATION;
        let values = vec![];
        let material_properties = properties_from_mask(properties, values);
        assert_eq!(None, material_properties.specular);
        assert_eq!(None, material_properties.roughness);
        assert_eq!(None, material_properties.attenuation);
    }

    #[test]
    fn can_parse_material_properties() {
        let bytes = include_bytes!("../resources/valid_material_properties.bytes").to_vec();
        let result = super::parse_material_properties(&bytes);
        assert!(result.is_done());
        let (_, material_properties) = result.unwrap();
        assert_eq!(Some(0.0), material_properties.plastic);
        assert_eq!(Some(0.189474), material_properties.roughness);
        assert_eq!(Some(0.5), material_properties.specular);
        assert_eq!(Some(0.547368), material_properties.ior);
        assert_eq!(Some(0.021053), material_properties.attenuation);
    }
}
