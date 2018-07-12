use {DEFAULT_PALETTE, DotVoxData, Material, material, Model, model, palette, Size, Voxel};
use nom::{IResult, le_u32};

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
                    },
                    Chunk::Pack(model) => models.push(model),
                    Chunk::Palette(palette) => palette_holder = palette,
                    Chunk::Material(material) => materials.push(material),
                    _ => println!("Unmapped chunk {:?}", chunk)
                }
            }

            DotVoxData {
                version,
                models,
                palette: palette_holder,
                materials,
            }
        },
        _ => DotVoxData {
            version: version,
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
            "MATT" => build_material_chunk(chunk_content),
            _ => {
                println!("Unknown childless chunk {:?}", id);
                Chunk::Unknown(id.to_owned())
            }
        }
    } else {
        let result: IResult<&[u8], Vec<Chunk>> = many0!(child_content, parse_chunk);
        let child_chunks = match result {
            IResult::Done(_, result) => result,
            _ => vec![]
        };
        match id {
            "MAIN" => Chunk::Main(child_chunks),
            "PACK" => build_pack_chunk(chunk_content),
            _ => Chunk::Unknown(id.to_owned())
        }
    }
}

fn build_material_chunk(chunk_content: &[u8]) -> Chunk {
    if let IResult::Done(_, material) = material::parse_material(chunk_content) {
        return Chunk::Material(material);
    }
    Chunk::Invalid(chunk_content.to_vec())
}

fn build_palette_chunk(chunk_content: &[u8]) -> Chunk {
    if let IResult::Done(_, palette) = palette::extract_palette(chunk_content) {
        return Chunk::Palette(palette)
    }
    Chunk::Invalid(chunk_content.to_vec())
}

fn build_pack_chunk(chunk_content: &[u8]) -> Chunk {
    if let IResult::Done(chunk_content, Chunk::Size(size)) = parse_chunk(chunk_content) {
        if let IResult::Done(_, Chunk::Voxels(voxels)) = parse_chunk(chunk_content) {
            return Chunk::Pack(Model { size, voxels: voxels.to_vec() })
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

#[cfg(test)]
mod tests {
    use super::*;
    use avow::vec;
    use {MaterialType, MaterialProperties};

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
                vec![Voxel::new(0, 0, 0, 225),
                     Voxel::new(0, 1, 1, 215),
                     Voxel::new(1, 0, 1, 235),
                     Voxel::new(1, 1, 0, 5),
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
    fn can_parse_material_chunk() {
        let bytes = include_bytes!("resources/valid_material.bytes").to_vec();
        let result = parse_chunk(&bytes);
        assert!(result.is_done());
        let (_, material) = result.unwrap();
        match material {
            Chunk::Material(material) => {
                assert_eq!(248, material.id);
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
            },
            chunk => panic!("Expecting Material chunk, got {:?}", chunk)
        };
    }

//    #[test]
//    fn can_parse_multiple_materials() {
//        let bytes = include_bytes!("resources/multi-materials.bytes").to_vec();
//        let result = parse_chunk(&bytes);
//        assert!(result.is_done());
//        let (_, materials) = result.unwrap();
//        vec::are_eq(
//            materials,
//            vec![
//                Material {
//                    id: 78,
//                    material_type: MaterialType::Metal(1.0),
//                    properties: MaterialProperties {
//                        plastic: Some(0.0),
//                        roughness: Some(0.1),
//                        specular: Some(0.5),
//                        ior: Some(0.3),
//                        ..Default::default()
//                    },
//                },
//                Material {
//                    id: 84,
//                    material_type: MaterialType::Metal(0.526316),
//                    properties: MaterialProperties {
//                        plastic: Some(0.0),
//                        roughness: Some(0.252632),
//                        specular: Some(0.736842),
//                        ior: Some(0.3),
//                        ..Default::default()
//                    },
//                },
//                Material {
//                    id: 248,
//                    material_type: MaterialType::Glass(0.810526),
//                    properties: MaterialProperties {
//                        plastic: Some(0.0),
//                        roughness: Some(0.189474),
//                        specular: Some(0.5),
//                        ior: Some(0.547368),
//                        attenuation: Some(0.021053),
//                        ..Default::default()
//                    },
//                },
//            ],
//        );
//    }
}
