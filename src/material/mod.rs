pub mod material_properties;
pub mod material_type;

use {MaterialProperties, MaterialType};
use material::material_type::parse_material_type;
use material::material_properties::parse_material_properties;
use nom::le_u32;

/// A material used to render this model.
#[derive(Debug, PartialEq)]
pub struct Material {
    /// The index into the color palette to apply this material to. As with the indices in the color
    /// palette used on the Voxels, this value has been corrected to match the in-memory indices of
    /// the color palette (i.e. is 1 less than the value stored in the file).
    pub id: u8,
    /// The type of material this is
    pub material_type: MaterialType,
    /// Additional property values.
    pub properties: MaterialProperties,
}

named!(pub parse_material <&[u8], Material>, do_parse!(
    id: le_u32 >>
    material_type: parse_material_type >>
    properties: parse_material_properties >>
    (Material {
      id: (id as u8).saturating_sub(1),
      material_type: material_type,
      properties: properties
    })
));
