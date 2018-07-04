use dot_vox_data::DotVoxData;
use material::extract_materials;
use model::extract_models;
use nom::le_u32;
use palette::{DEFAULT_PALETTE, extract_palette};

const MAGIC_NUMBER: &'static str = "VOX ";

named!(pub parse_vox_file <&[u8], DotVoxData>, do_parse!(
  tag!(MAGIC_NUMBER) >>
  version: le_u32  >>
  take!(12)          >>
  models: extract_models >>
  palette: opt_res!(extract_palette) >>
  opt_res!(complete!(take!(4))) >>
  materials: opt_res!(extract_materials) >>
  (DotVoxData {
    version: version,
    models: models,
    palette: palette.unwrap_or_else(|_| DEFAULT_PALETTE.to_vec()),
    materials: materials.unwrap_or_else(|_| vec![]),
  })
));