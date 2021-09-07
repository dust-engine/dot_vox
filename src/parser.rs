use {DEFAULT_PALETTE, DotVoxData, Model, model, palette, Size, Voxel};
use nom::bytes::complete::{tag, take};
use nom::combinator::{flat_map, map_res};
use nom::multi::{fold_many_m_n, many0};
use nom::number::complete::le_u32;
use nom::sequence::pair;
use nom::IResult;
use std::collections::HashMap;
use std::str;
use std::str::Utf8Error;

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

pub fn to_str(i: &[u8]) -> Result<String, Utf8Error> {
    let res = str::from_utf8(i)?;
    Ok(res.to_owned())
}

pub fn parse_vox_file(i: &[u8]) -> IResult<&[u8], DotVoxData> {
    let (i, _) = tag(MAGIC_NUMBER)(i)?;
    let (i, version) = le_u32(i)?;
    let (i, main) = parse_chunk(i)?;
    Ok((i, map_chunk_to_data(version, main)))
}

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

fn parse_chunk(i: &[u8]) -> IResult<&[u8], Chunk> {
    let (i, id) = map_res(take(4usize), str::from_utf8)(i)?;
    let (i, (content_size, children_size)) = pair(le_u32, le_u32)(i)?;
    let (i, chunk_content) = take(content_size)(i)?;
    let (i, child_content) = take(children_size)(i)?;
    let chunk = build_chunk(id, chunk_content, children_size, child_content);
    Ok((i, chunk))
}

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
        let result: IResult<&[u8], Vec<Chunk>> = many0(parse_chunk)(child_content);
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

fn build_material_chunk(chunk_content: &[u8]) -> Chunk {
    if let Ok((_, material)) = parse_material(chunk_content) {
        return Chunk::Material(material);
    }
    Chunk::Invalid(chunk_content.to_vec())
}

fn build_palette_chunk(chunk_content: &[u8]) -> Chunk {
    if let Ok((_, palette)) = palette::extract_palette(chunk_content) {
        return Chunk::Palette(palette);
    }
    Chunk::Invalid(chunk_content.to_vec())
}

fn build_pack_chunk(chunk_content: &[u8]) -> Chunk {
    if let Ok((chunk_content, Chunk::Size(size))) = parse_chunk(chunk_content) {
        if let Ok((_, Chunk::Voxels(voxels))) = parse_chunk(chunk_content) {
            return Chunk::Pack(Model { size, voxels: voxels.to_vec() });
        }
    }
    Chunk::Invalid(chunk_content.to_vec())
}

fn build_size_chunk(chunk_content: &[u8]) -> Chunk {
    match model::parse_size(chunk_content) {
        Ok((_, size)) => Chunk::Size(size),
        _ => Chunk::Invalid(chunk_content.to_vec())
    }
}

fn build_voxel_chunk(chunk_content: &[u8]) -> Chunk {
    match model::parse_voxels(chunk_content) {
        Ok((_, voxels)) => Chunk::Voxels(voxels),
        _ => Chunk::Invalid(chunk_content.to_vec())
    }
}

pub fn parse_material(i: &[u8]) -> IResult<&[u8], Material> {
    let (i, (id, properties)) = pair(le_u32, parse_dict)(i)?;
    Ok((i, Material { id, properties }))
}

fn parse_dict(i: &[u8]) -> IResult<&[u8], Dict> {
    let (i, n) = le_u32(i)?;
    let init = move || Dict::with_capacity(n as usize);
    let fold = |mut map: Dict, (key, value)| {
        map.insert(key, value);
        map
    };
    fold_many_m_n(n as usize, n as usize, parse_dict_entry, init, fold)(i)
}

fn parse_dict_entry(i: &[u8]) -> IResult<&[u8], (String, String)> {
    pair(parse_string, parse_string)(i)
}

fn parse_string(i: &[u8]) -> IResult<&[u8], String> {
    let bytes = flat_map(le_u32, take);
    map_res(bytes, to_str)(i)
}

#[cfg(test)]
mod tests {
    use avow::vec;
    use super::*;

    #[test]
    fn can_parse_size_chunk() {
        let bytes = include_bytes!("resources/valid_size.bytes").to_vec();
        let result = parse_chunk(&bytes);
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
        let result = parse_chunk(&bytes);
        assert!(result.is_ok());
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
        assert!(result.is_ok());
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
            Ok((_, material)) => {
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
