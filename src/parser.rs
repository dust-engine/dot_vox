use nom::types::CompleteByteSlice;
use nom::IResult;
use std::collections::HashMap;
use std::str;
use std::str::Utf8Error;
use {
    model, palette, scene, DotVoxData, Layer, Model, SceneGroup, SceneNode, SceneShape,
    SceneTransform, Size, Voxel, DEFAULT_PALETTE,
};

const MAGIC_NUMBER: &'static str = "VOX ";

#[derive(Debug, PartialEq)]
pub enum Chunk {
    Main(Vec<Chunk>),
    Size(Size),
    Voxels(Vec<Voxel>),
    Pack(Model),
    Palette(Vec<u32>),
    Material(Material),
    TransformNode(SceneTransform),
    GroupNode(SceneGroup),
    ShapeNode(SceneShape),
    Layer(Layer),
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

/// Recognizes an unsigned 1 byte integer (equivalent to take!(1)
#[inline]
pub fn le_u8(i: CompleteByteSlice) -> IResult<CompleteByteSlice, u8> {
    Ok((CompleteByteSlice(&i[1..]), i[0]))
}

/// Recognizes little endian unsigned 4 bytes integer
#[inline]
pub fn le_u32(i: CompleteByteSlice) -> IResult<CompleteByteSlice, u32> {
    let res = ((i[3] as u32) << 24) + ((i[2] as u32) << 16) + ((i[1] as u32) << 8) + i[0] as u32;
    Ok((CompleteByteSlice(&i[4..]), res))
}

/// Recognizes little endian signed 4 bytes integer
#[inline]
pub fn le_i32(i: CompleteByteSlice) -> IResult<CompleteByteSlice, i32> {
    match le_u32(i) {
        Ok(result) => Ok((CompleteByteSlice(&i[4..]), result.1 as i32)),
        Err(e) => Err(e),
    }
}

pub fn to_str(i: CompleteByteSlice) -> Result<String, Utf8Error> {
    let res = str::from_utf8(i.0)?;
    Ok(res.to_owned())
}

named!(pub parse_vox_file <CompleteByteSlice, DotVoxData>, do_parse!(
  tag!(MAGIC_NUMBER) >>
  version: le_u32 >>
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
            let mut scene: Vec<SceneNode> = vec![];
            let mut layers: Vec<Dict> = vec![];

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
                    Chunk::TransformNode(scene_transform) => {
                        scene.push(SceneNode::Transform {
                            attributes: scene_transform.header.attributes,
                            frames: scene_transform.frames,
                            child: scene_transform.child,
                        });
                    }
                    Chunk::GroupNode(scene_group) => scene.push(SceneNode::Group {
                        attributes: scene_group.header.attributes,
                        children: scene_group.children,
                    }),
                    Chunk::ShapeNode(scene_shape) => scene.push(SceneNode::Shape {
                        attributes: scene_shape.header.attributes,
                        models: scene_shape.models,
                    }),
                    Chunk::Layer(layer) => layers.push(layer.attributes),
                    _ => debug!("Unmapped chunk {:?}", chunk),
                }
            }

            DotVoxData {
                version,
                models,
                palette: palette_holder,
                materials,
                scene,
                layers,
            }
        }
        _ => DotVoxData {
            version,
            models: vec![],
            palette: vec![],
            materials: vec![],
            scene: vec![],
            layers: vec![],
        },
    }
}

named!(parse_chunk <CompleteByteSlice, Chunk>, do_parse!(
    id: map_res!(take!(4), to_str) >>
    content_size: le_u32 >>
    children_size: le_u32 >>
    chunk_content: take!(content_size) >>
    child_content: take!(children_size) >>
    (build_chunk(id, chunk_content, children_size, child_content))
));

fn build_chunk(
    string: String,
    chunk_content: CompleteByteSlice,
    children_size: u32,
    child_content: CompleteByteSlice,
) -> Chunk {
    let id = string.as_str();
    if children_size == 0 {
        match id {
            "SIZE" => build_size_chunk(chunk_content),
            "XYZI" => build_voxel_chunk(chunk_content),
            "PACK" => build_pack_chunk(chunk_content),
            "RGBA" => build_palette_chunk(chunk_content),
            "MATL" => build_material_chunk(chunk_content),
            "nTRN" => build_scene_transform_chunk(chunk_content),
            "nGRP" => build_scene_group_chunk(chunk_content),
            "nSHP" => build_scene_shape_chunk(chunk_content),
            "LAYR" => build_layer_chunk(chunk_content),
            _ => {
                debug!("Unknown childless chunk {:?}", id);
                Chunk::Unknown(id.to_owned())
            }
        }
    } else {
        let result: IResult<CompleteByteSlice, Vec<Chunk>> = many0!(child_content, parse_chunk);
        let child_chunks = match result {
            Ok((_, result)) => result,
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

fn build_material_chunk(chunk_content: CompleteByteSlice) -> Chunk {
    if let Ok((_, material)) = parse_material(chunk_content) {
        return Chunk::Material(material);
    }
    Chunk::Invalid(chunk_content.to_vec())
}

fn build_palette_chunk(chunk_content: CompleteByteSlice) -> Chunk {
    if let Ok((_, palette)) = palette::extract_palette(chunk_content) {
        return Chunk::Palette(palette);
    }
    Chunk::Invalid(chunk_content.to_vec())
}

fn build_pack_chunk(chunk_content: CompleteByteSlice) -> Chunk {
    if let Ok((chunk_content, Chunk::Size(size))) = parse_chunk(chunk_content) {
        if let Ok((_, Chunk::Voxels(voxels))) = parse_chunk(chunk_content) {
            return Chunk::Pack(Model {
                size,
                voxels: voxels.to_vec(),
            });
        }
    }
    Chunk::Invalid(chunk_content.to_vec())
}

fn build_size_chunk(chunk_content: CompleteByteSlice) -> Chunk {
    match model::parse_size(chunk_content) {
        Ok((_, size)) => Chunk::Size(size),
        _ => Chunk::Invalid(chunk_content.to_vec()),
    }
}

fn build_voxel_chunk(chunk_content: CompleteByteSlice) -> Chunk {
    match model::parse_voxels(chunk_content) {
        Ok((_, voxels)) => Chunk::Voxels(voxels),
        _ => Chunk::Invalid(chunk_content.to_vec()),
    }
}

fn build_scene_transform_chunk(chunk_content: CompleteByteSlice) -> Chunk {
    match scene::parse_scene_transform(chunk_content) {
        Ok((_, transform_node)) => Chunk::TransformNode(transform_node),
        _ => Chunk::Invalid(chunk_content.to_vec()),
    }
}

fn build_scene_group_chunk(chunk_content: CompleteByteSlice) -> Chunk {
    match scene::parse_scene_group(chunk_content) {
        Ok((_, group_node)) => Chunk::GroupNode(group_node),
        _ => Chunk::Invalid(chunk_content.to_vec()),
    }
}

fn build_scene_shape_chunk(chunk_content: CompleteByteSlice) -> Chunk {
    match scene::parse_scene_shape(chunk_content) {
        Ok((_, shape_node)) => Chunk::ShapeNode(shape_node),
        _ => Chunk::Invalid(chunk_content.to_vec()),
    }
}

fn build_layer_chunk(chunk_content: CompleteByteSlice) -> Chunk {
    match scene::parse_layer(chunk_content) {
        Ok((_, layer)) => Chunk::Layer(layer),
        _ => Chunk::Invalid(chunk_content.to_vec()),
    }
}

named!(pub parse_material <CompleteByteSlice, Material>, do_parse!(
    id: le_u32 >>
    properties: parse_dict >>
    (Material { id, properties })
));

named!(pub parse_dict <CompleteByteSlice, Dict>, do_parse!(
    count: le_u32 >>
    entries: many_m_n!(count as usize, count as usize, parse_dict_entry) >>
    (build_dict_from_entries(entries))
));

named!(parse_dict_entry <CompleteByteSlice, (String, String)>, tuple!(parse_string, parse_string));

named!(parse_string <CompleteByteSlice, String>, do_parse!(
    count: le_u32 >>
    buffer: map_res!(take!(count), to_str) >>
    (buffer)
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
    use super::*;
    use avow::vec;

    #[test]
    fn can_parse_size_chunk() {
        let bytes = include_bytes!("resources/valid_size.bytes").to_vec();
        let result = parse_chunk(CompleteByteSlice(&bytes));
        assert!(result.is_ok());
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
        let result = parse_chunk(CompleteByteSlice(&bytes));
        assert!(result.is_ok());
        let (_, voxels) = result.unwrap();
        match voxels {
            Chunk::Voxels(voxels) => vec::are_eq(
                voxels,
                vec![
                    Voxel {
                        x: 0,
                        y: 0,
                        z: 0,
                        i: 225,
                    },
                    Voxel {
                        x: 0,
                        y: 1,
                        z: 1,
                        i: 215,
                    },
                    Voxel {
                        x: 1,
                        y: 0,
                        z: 1,
                        i: 235,
                    },
                    Voxel {
                        x: 1,
                        y: 1,
                        z: 0,
                        i: 5,
                    },
                ],
            ),
            chunk => panic!("Expecting Voxel chunk, got {:?}", chunk),
        };
    }

    #[test]
    fn can_parse_palette_chunk() {
        let bytes = include_bytes!("resources/valid_palette.bytes").to_vec();
        let result = parse_chunk(CompleteByteSlice(&bytes));
        assert!(result.is_ok());
        let (_, palette) = result.unwrap();
        match palette {
            Chunk::Palette(palette) => vec::are_eq(palette, DEFAULT_PALETTE.to_vec()),
            chunk => panic!("Expecting Palette chunk, got {:?}", chunk),
        };
    }

    #[test]
    fn can_parse_a_material_chunk() {
        let bytes = include_bytes!("resources/valid_material.bytes").to_vec();
        let result = parse_material(CompleteByteSlice(&bytes));
        match result {
            Ok((_, material)) => {
                assert_eq!(material.id, 0);
                assert_eq!(
                    material.properties.get("_type"),
                    Some(&"_diffuse".to_owned())
                );
                assert_eq!(material.properties.get("_weight"), Some(&"1".to_owned()));
                assert_eq!(material.properties.get("_rough"), Some(&"0.1".to_owned()));
                assert_eq!(material.properties.get("_spec"), Some(&"0.5".to_owned()));
                assert_eq!(material.properties.get("_ior"), Some(&"0.3".to_owned()));
            }
            _ => panic!("Expected Done, got {:?}", result),
        }
    }
}
