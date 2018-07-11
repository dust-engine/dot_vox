use dot_vox_data::DotVoxData;
use model::size::parse_size;
use model::voxel::parse_voxels;
use nom::{IResult, le_u32};
use palette::DEFAULT_PALETTE;
use Size;
use Voxel;
use Model;

const MAGIC_NUMBER: &'static str = "VOX ";

#[derive(Debug, PartialEq)]
pub enum Chunk {
    Main(Vec<Chunk>),
    Size(Size),
    Voxels(Vec<Voxel>),
    Unknown(String),
    Invalid(Vec<u8>),
}

named!(pub parse_vox_file <&[u8], DotVoxData>, do_parse!(
  tag!(MAGIC_NUMBER) >>
  version: le_u32  >>
  main: parse_chunk >>
//  models: extract_models >>
//  palette: opt_res!(extract_palette) >>
//  opt_res!(complete!(take!(4))) >>
//  materials: opt_res!(extract_materials) >>
  (map_chunk_to_data(version, main))
));

fn map_chunk_to_data(version: u32, main: Chunk) -> DotVoxData {
    match main {
        Chunk::Main(children) => {
            let mut size_holder: Option<Size> = None;
            let mut models: Vec<Model> = vec![];
            for chunk in children {
                match chunk {
                    Chunk::Size(size) => size_holder = Some(size),
                    Chunk::Voxels(voxels) => {
                        if let Some(size) = size_holder {
                            models.push(Model { size, voxels })
                        }
                    },
                    _ => println!("Unmapped chunk {:?}", chunk)
                }
            }

            DotVoxData {
                version,
                models,
                palette: vec![],
                materials: vec![],
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
            _ => Chunk::Unknown(id.to_owned())
        }
    } else {
        let result: IResult<&[u8], Vec<Chunk>> = many0!(child_content, parse_chunk);
        let child_chunks = match result {
            IResult::Done(_, result) => result,
            _ => vec![]
        };
        match id {
            "MAIN" => Chunk::Main(child_chunks),
            _ => Chunk::Unknown(id.to_owned())
        }
    }
}

fn build_size_chunk(chunk_content: &[u8]) -> Chunk {
    match parse_size(chunk_content) {
        IResult::Done(_, size) => Chunk::Size(size),
        _ => Chunk::Invalid(chunk_content.to_vec())
    }
}

fn build_voxel_chunk(chunk_content: &[u8]) -> Chunk {
    match parse_voxels(chunk_content) {
        IResult::Done(_, voxels) => Chunk::Voxels(voxels),
        _ => Chunk::Invalid(chunk_content.to_vec())
    }
}

#[test]
fn panic_to_show_format() {
    let bytes = include_bytes!("resources/placeholder.vox").to_vec();
    let result = parse_vox_file(&bytes);
    panic!("{:?}", result)
}
