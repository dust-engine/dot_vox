use {Size, take_u32, Voxel};
use model::size::parse_size;
use model::voxel::parse_voxels;

/// A renderable voxel Model
#[derive(Debug, PartialEq)]
pub struct Model {
    /// The size of the model in voxels
    pub size: Size,
    /// The voxels to be displayed.
    pub voxels: Vec<Voxel>,
}

named!(parse_model <&[u8], Model>, do_parse!(
  size: parse_size     >>
  voxels: parse_voxels >>
  (Model { size: size, voxels: voxels })
));

named!(parse_models <&[u8], Vec<Model> >, do_parse!(
  take!(12)             >>
  model_count: take_u32 >>
  models: many_m_n!(model_count as usize, model_count as usize, parse_model) >>
  (models)
));

named!(pub extract_models <&[u8], Vec<Model> >, switch!(peek!(take!(4)),
    b"PACK" => call!(parse_models) |
    b"SIZE" => map!(call!(parse_model), |m| vec!(m))
));

//TODO add model tests here