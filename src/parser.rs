use crate::{
    model, palette, scene, Color, DotVoxData, Frame, Layer, Model, RawLayer, SceneGroup, SceneNode,
    SceneShape, SceneTransform, Size, Voxel, DEFAULT_PALETTE,
};
use nom::{
    bytes::complete::{tag, take},
    combinator::{flat_map, map_res},
    error::make_error,
    multi::{fold_many_m_n, many0},
    number::complete::le_u32,
    sequence::pair,
    IResult,
};
use std::{mem::size_of, str, str::Utf8Error};

#[cfg(feature = "ahash")]
use ahash::AHashMap as HashMap;

#[cfg(not(feature = "ahash"))]
use std::collections::HashMap;

const MAGIC_NUMBER: &str = "VOX ";

#[derive(Debug, PartialEq)]
pub enum Chunk {
    Main(Vec<Chunk>),
    Size(Size),
    Voxels(Vec<Voxel>),
    Palette(Vec<Color>),
    Material(Material),
    TransformNode(SceneTransform),
    GroupNode(SceneGroup),
    ShapeNode(SceneShape),
    Layer(RawLayer),
    Unknown(String),
    Invalid(Vec<u8>),
}

/// A material used to render this model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Material {
    /// The Material's ID.  Corresponds to an index in the palette.
    pub id: u32,
    /// Properties of the material, mapped by property name.
    pub properties: Dict,
}

// TODO: maybe material schemas?
impl Material {
    /// The `_type` field, if present
    pub fn material_type(&self) -> Option<&str> {
        if let Some(t) = self.properties.get("_type") {
            return Some(t.as_str());
        }

        None
    }

    /// The `_weight` field associated with the material
    pub fn weight(&self) -> Option<f32> {
        let w = self.get_f32("_weight");

        if let Some(w) = w {
            if !(0.0..=1.0).contains(&w) {
                debug!("_weight observed outside of range of [0..1]: {}", w);
            }
        }

        w
    }

    /// The `_metal` field associated with the material
    pub fn metalness(&self) -> Option<f32> {
        self.get_f32("_metal")
    }

    /// The `_rough` field associated with the material
    pub fn roughness(&self) -> Option<f32> {
        self.get_f32("_rough")
    }

    /// The `_sp` field associated with the material.
    pub fn specular(&self) -> Option<f32> {
        self.get_f32("_sp")
    }

    /// The `_ior` field associated with the material
    pub fn refractive_index(&self) -> Option<f32> {
        self.get_f32("_ior")
    }

    /// The `_emit` field associated with the material
    pub fn emission(&self) -> Option<f32> {
        self.get_f32("_emit")
    }

    /// The '_ldr' field associated with the material. This is a 'hack' factor
    /// to scale emissive materials visually by so they look less bright when
    /// rendered. I.e. this blends between the actual color of the resp.
    /// voxel (`low_dynamic_range_scale` = 0) and its pure diffuse color
    /// (`low_dynamic_range_scale` = 1)
    pub fn low_dynamic_range_scale(&self) -> Option<f32> {
        self.get_f32("_ldr")
    }

    /// The '_ri' field associated with the material (appears to just be 1 +
    /// _ior)
    pub fn ri(&self) -> Option<f32> {
        self.get_f32("_ior")
    }

    /// The `_att` field associated with the `glass` material.
    ///
    /// This is the falloff that models the optiocal density of the medium.
    pub fn attenuation(&self) -> Option<f32> {
        self.get_f32("_att")
    }

    /// The `_flux` field associated with the `emissive` material.
    pub fn radiant_flux(&self) -> Option<f32> {
        self.get_f32("_flux")
    }

    /// The `_g` field associated with the material.
    pub fn phase(&self) -> Option<f32> {
        self.get_f32("_g")
    }

    /// The `_alpha` field associated with the material.
    ///
    /// This is the alpha/blending value that is used to blend the voxel with
    /// the background (compositing related, has no relation to physics).
    pub fn opacity(&self) -> Option<f32> {
        self.get_f32("_alpha")
    }

    /// The `_trans` field associated with the material.
    ///
    /// This is the transparency of the material. I.e. a physical property,
    /// honours [`refractive_index()`](Material::refractive_index), see above.
    pub fn transparency(&self) -> Option<f32> {
        self.get_f32("_trans")
    }

    /// The `_d` field associated with the `cloud` material.
    ///
    /// This is the density of the volumetric medium.
    pub fn density(&self) -> Option<f32> {
        self.get_f32("_d")
    }

    /// The `_media` field associated with the material.
    /// This corresponds to the `cloud` material.
    pub fn media(&self) -> Option<f32> {
        self.get_f32("_media")
    }

    /// The `_media_type` field associated with the material.
    ///
    /// Corresponds to the type of `cloud`: `absorb`, `scatter`, `emissive`,
    /// `subsurface scattering`
    pub fn media_type(&self) -> Option<&str> {
        if let Some(t) = self.properties.get("_media_type") {
            return Some(t.as_str());
        }

        None
    }

    fn get_f32(&self, prop: &str) -> Option<f32> {
        if let Some(t) = self.properties.get(prop) {
            match t.parse::<f32>() {
                Ok(x) => return Some(x),
                Err(_) => {
                    debug!("Could not parse float for property '{}': {}", prop, t)
                }
            }
        }

        None
    }
}

/// General dictionary.
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
            let mut palette_holder: Vec<Color> = DEFAULT_PALETTE.to_vec();
            let mut materials: Vec<Material> = vec![];
            let mut scene: Vec<SceneNode> = vec![];
            let mut layers: Vec<Layer> = Vec::new();

            for chunk in children {
                match chunk {
                    Chunk::Size(size) => size_holder = Some(size),
                    Chunk::Voxels(voxels) => {
                        if let Some(size) = size_holder {
                            models.push(Model { size, voxels })
                        }
                    }
                    Chunk::Palette(palette) => palette_holder = palette,
                    Chunk::Material(material) => materials.push(material),
                    Chunk::TransformNode(scene_transform) => {
                        scene.push(SceneNode::Transform {
                            attributes: scene_transform.header.attributes,
                            frames: scene_transform.frames.into_iter().map(Frame::new).collect(),
                            child: scene_transform.child,
                            layer_id: scene_transform.layer_id,
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
                    Chunk::Layer(layer) => {
                        if layer.id as usize != layers.len() {
                            // Not sure if this actually happens in practice, but nothing in the
                            // spec prohibits it.
                            debug!(
                                "Unexpected layer id {} encountered, layers may be out of order.",
                                layer.id
                            );
                        }

                        layers.push(Layer {
                            attributes: layer.attributes,
                        });
                    }
                    _ => debug!("Unmapped chunk {:?}", chunk),
                }
            }

            DotVoxData {
                version,
                models,
                palette: palette_holder,
                materials,
                scenes: scene,
                layers,
            }
        }
        _ => DotVoxData {
            version,
            models: vec![],
            palette: vec![],
            materials: vec![],
            scenes: vec![],
            layers: vec![],
        },
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

fn build_chunk(id: &str, chunk_content: &[u8], children_size: u32, child_content: &[u8]) -> Chunk {
    if children_size == 0 {
        match id {
            "SIZE" => build_size_chunk(chunk_content),
            "XYZI" => build_voxel_chunk(chunk_content),
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

fn build_size_chunk(chunk_content: &[u8]) -> Chunk {
    match model::parse_size(chunk_content) {
        Ok((_, size)) => Chunk::Size(size),
        _ => Chunk::Invalid(chunk_content.to_vec()),
    }
}

fn build_voxel_chunk(chunk_content: &[u8]) -> Chunk {
    match model::parse_voxels(chunk_content) {
        Ok((_, voxels)) => Chunk::Voxels(voxels),
        _ => Chunk::Invalid(chunk_content.to_vec()),
    }
}

fn build_scene_transform_chunk(chunk_content: &[u8]) -> Chunk {
    match scene::parse_scene_transform(chunk_content) {
        Ok((_, transform_node)) => Chunk::TransformNode(transform_node),
        _ => Chunk::Invalid(chunk_content.to_vec()),
    }
}

fn build_scene_group_chunk(chunk_content: &[u8]) -> Chunk {
    match scene::parse_scene_group(chunk_content) {
        Ok((_, group_node)) => Chunk::GroupNode(group_node),
        _ => Chunk::Invalid(chunk_content.to_vec()),
    }
}

fn build_scene_shape_chunk(chunk_content: &[u8]) -> Chunk {
    match scene::parse_scene_shape(chunk_content) {
        Ok((_, shape_node)) => Chunk::ShapeNode(shape_node),
        _ => Chunk::Invalid(chunk_content.to_vec()),
    }
}

fn build_layer_chunk(chunk_content: &[u8]) -> Chunk {
    match scene::parse_layer(chunk_content) {
        Ok((_, layer)) => Chunk::Layer(layer),
        _ => Chunk::Invalid(chunk_content.to_vec()),
    }
}

pub fn parse_material(i: &[u8]) -> IResult<&[u8], Material> {
    let (i, (id, properties)) = pair(le_u32, parse_dict)(i)?;
    Ok((i, Material { id, properties }))
}

pub(crate) fn parse_dict(i: &[u8]) -> IResult<&[u8], Dict> {
    let (i, n) = le_u32(i)?;
    let n = validate_count(i, n, size_of::<u32>() * 2)?;

    let init = move || Dict::with_capacity(n);
    let fold = |mut map: Dict, (key, value)| {
        map.insert(key, value);
        map
    };
    fold_many_m_n(n, n, parse_dict_entry, init, fold)(i)
}

fn parse_dict_entry(i: &[u8]) -> IResult<&[u8], (String, String)> {
    pair(parse_string, parse_string)(i)
}

fn parse_string(i: &[u8]) -> IResult<&[u8], String> {
    let bytes = flat_map(le_u32, take);
    map_res(bytes, to_str)(i)
}

/// Validate that a given count of items is possible to achieve given the size of the
/// input, then convert it to [`usize`].
///
/// This ensures that parsing an invalid/malicious file cannot cause excessive memory
/// allocation in the form of data structures' capacity. It should be used whenever
/// [`nom::multi::count`] or a `with_capacity()` function is used with a count taken from
/// the file.
///
/// `minimum_object_size` must not be smaller than the minimum possible size of a parsed
/// value.
pub(crate) fn validate_count(
    i: &[u8],
    count: u32,
    minimum_object_size: usize,
) -> Result<usize, nom::Err<nom::error::Error<&[u8]>>> {
    let Ok(count) = usize::try_from(count) else {
        return Err(nom::Err::Failure(make_error(i, nom::error::ErrorKind::TooLarge)));
    };

    if count > i.len() / minimum_object_size {
        Err(nom::Err::Failure(make_error(
            i,
            nom::error::ErrorKind::TooLarge,
        )))
    } else {
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use avow::vec;

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
        let result = parse_chunk(&bytes);
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
        let result = parse_material(&bytes);
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
