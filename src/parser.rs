use {DEFAULT_PALETTE, DotVoxData, Model, model, palette, Size, Voxel};
use nom::{IResult, le_u32};
use std::collections::HashMap;

const MAGIC_NUMBER: &'static str = "VOX ";

#[derive(Debug, PartialEq)]
pub enum Chunk {
    Main(Vec<Chunk>),
    Size(Size),
    Voxels(Vec<Voxel>),
    Pack(Model),
    Palette(Vec<u32>),
    Material(Material),
    Unknown(String),
    Invalid(Vec<u8>),
}

/// A material used to render this model.
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    /// The Material's ID
    pub id: u32,
    /// Properties of the material, mapped by property name.
    pub properties: Dict,
}

/// General dictionary
pub type Dict = HashMap<String, String>;

named!(pub parse_vox_file <&[u8], DotVoxData>, do_parse!(
  tag!(MAGIC_NUMBER) >>
  version: le_u32  >>
  main: parse_chunk >>
  (map_chunk_to_data(version, main))
));

fn map_chunk_to_data(version: u32, main: Chunk) -> DotVoxData {
    match main {
        Chunk::Main(children) => {
            let mut size_holder: Option<Size> = None;
            let mut models: Vec<Model> = vec![];
            let mut palette_holder: Vec<u32> = DEFAULT_PALETTE.to_vec();
            let mut materials: Vec<Material> = vec![];
            for chunk in children {
                match chunk {
                    Chunk::Size(size) => size_holder = Some(size),
                    Chunk::Voxels(voxels) => {
                        if let Some(size) = size_holder {
                            models.push(Model { size, voxels })
                        }
                    }
                    Chunk::Pack(model) => models.push(model),
                    Chunk::Palette(palette) => palette_holder = palette,
                    Chunk::Material(material) => materials.push(material),
                    _ => debug!("Unmapped chunk {:?}", chunk)
                }
            }

            DotVoxData {
                version,
                models,
                palette: palette_holder,
                materials,
            }
        }
        _ => DotVoxData {
            version,
            models: vec![],
            palette: vec![],
            materials: vec![],
        }
    }
}

named!(parse_chunk <&[u8], Chunk>, do_parse!(
    id: take_str!(4) >>
    content_size: le_u32 >>
    children_size: le_u32 >>
    chunk_content: take!(content_size) >>
    child_content: take!(children_size) >>
    (build_chunk(id, chunk_content, children_size, child_content))
));

fn build_chunk(id: &str,
               chunk_content: &[u8],
               children_size: u32,
               child_content: &[u8]) -> Chunk {
    if children_size == 0 {
        match id {
            "SIZE" => build_size_chunk(chunk_content),
            "XYZI" => build_voxel_chunk(chunk_content),
            "PACK" => build_pack_chunk(chunk_content),
            "RGBA" => build_palette_chunk(chunk_content),
            "MATL" => build_material_chunk(chunk_content),
            _ => {
                debug!("Unknown childless chunk {:?}", id);
                Chunk::Unknown(id.to_owned())
            }
        }
    } else {
        let result: IResult<&[u8], Vec<Chunk>> = many0!(child_content, parse_chunk);
        let child_chunks = match result {
            IResult::Done(_, result) => result,
            result => {
                debug!("Failed to parse child chunks, due to {:?}", result);
                vec![]
            }
        };
        match id {
            "MAIN" => Chunk::Main(child_chunks),
            "PACK" => build_pack_chunk(chunk_content),
            _ => {
                debug!("Unknown chunk with children {:?}", id);
                Chunk::Unknown(id.to_owned())
            }
        }
    }
}

fn build_material_chunk(chunk_content: &[u8]) -> Chunk {
    if let IResult::Done(_, material) = parse_material(chunk_content) {
        return Chunk::Material(material);
    }
    Chunk::Invalid(chunk_content.to_vec())
}

fn build_palette_chunk(chunk_content: &[u8]) -> Chunk {
    if let IResult::Done(_, palette) = palette::extract_palette(chunk_content) {
        return Chunk::Palette(palette);
    }
    Chunk::Invalid(chunk_content.to_vec())
}

fn build_pack_chunk(chunk_content: &[u8]) -> Chunk {
    if let IResult::Done(chunk_content, Chunk::Size(size)) = parse_chunk(chunk_content) {
        if let IResult::Done(_, Chunk::Voxels(voxels)) = parse_chunk(chunk_content) {
            return Chunk::Pack(Model { size, voxels: voxels.to_vec() });
        }
    }
    Chunk::Invalid(chunk_content.to_vec())
}

fn build_size_chunk(chunk_content: &[u8]) -> Chunk {
    match model::parse_size(chunk_content) {
        IResult::Done(_, size) => Chunk::Size(size),
        _ => Chunk::Invalid(chunk_content.to_vec())
    }
}

fn build_voxel_chunk(chunk_content: &[u8]) -> Chunk {
    match model::parse_voxels(chunk_content) {
        IResult::Done(_, voxels) => Chunk::Voxels(voxels),
        _ => Chunk::Invalid(chunk_content.to_vec())
    }
}

named!(pub parse_material <&[u8], Material>, do_parse!(
    id: le_u32 >>
    properties: parse_dict >>
    (Material { id, properties })
));


named!(parse_dict <&[u8], Dict>, do_parse!(
    count: le_u32 >>
    entries: many_m_n!(count as usize, count as usize, parse_dict_entry) >>
    (build_dict_from_entries(entries))
));

named!(parse_dict_entry <&[u8], (String, String)>, tuple!(parse_string, parse_string));

named!(parse_string <&[u8], String>, do_parse!(
    count: le_u32 >>
    buffer: take_str!(count) >>
    (buffer.to_owned())
));

fn build_dict_from_entries(entries: Vec<(String, String)>) -> Dict {
    let mut map = HashMap::with_capacity(entries.len());
    for (key, value) in entries {
        map.insert(key, value);
    }
    map
}

#[cfg(test)]
mod tests {
    use avow::vec;
    use super::*;

    #[test]
    fn can_parse_size_chunk() {
        let bytes = include_bytes!("resources/valid_size.bytes").to_vec();
        let result = parse_chunk(&bytes);
        assert!(result.is_done());
        let (_, size) = result.unwrap();
        assert_eq!(
            size,
            Chunk::Size(Size {
                x: 24,
                y: 24,
                z: 24,
            })
        );
    }

    #[test]
    fn can_parse_voxels_chunk() {
        let bytes = include_bytes!("resources/valid_voxels.bytes").to_vec();
        let result = parse_chunk(&bytes);
        assert!(result.is_done());
        let (_, voxels) = result.unwrap();
        match voxels {
            Chunk::Voxels(voxels) => vec::are_eq(
                voxels,
                vec![Voxel { x: 0, y: 0, z: 0, i: 225 },
                     Voxel { x: 0, y: 1, z: 1, i: 215 },
                     Voxel { x: 1, y: 0, z: 1, i: 235 },
                     Voxel { x: 1, y: 1, z: 0, i: 5 },
                ],
            ),
            chunk => panic!("Expecting Voxel chunk, got {:?}", chunk)
        };
    }

    #[test]
    fn can_parse_palette_chunk() {
        let bytes = include_bytes!("resources/valid_palette.bytes").to_vec();
        let result = parse_chunk(&bytes);
        assert!(result.is_done());
        let (_, palette) = result.unwrap();
        match palette {
            Chunk::Palette(palette) => vec::are_eq(palette, DEFAULT_PALETTE.to_vec()),
            chunk => panic!("Expecting Palette chunk, got {:?}", chunk)
        };
    }

    #[test]
    fn can_parse_a_material_chunk() {
        let bytes = include_bytes!("resources/valid_material.bytes").to_vec();
        let result = parse_material(&bytes);
        match result {
            IResult::Done(_, material) => {
                assert_eq!(material.id, 0);
                assert_eq!(material.properties.get("_type"), Some(&"_diffuse".to_owned()));
                assert_eq!(material.properties.get("_weight"), Some(&"1".to_owned()));
                assert_eq!(material.properties.get("_rough"), Some(&"0.1".to_owned()));
                assert_eq!(material.properties.get("_spec"), Some(&"0.5".to_owned()));
                assert_eq!(material.properties.get("_ior"), Some(&"0.3".to_owned()));
            }
            _ => panic!("Expected Done, got {:?}", result)
        }
    }
}
