use {Material, Model};

/// Container for .vox file data
#[derive(Debug, PartialEq)]
pub struct DotVoxData {
    /// The version number of the .vox file.
    pub version: u32,
    /// A Vec of all the models contained within this file.
    pub models: Vec<Model>,
    /// A Vec containing the colour palette as 32-bit integers
    pub palette: Vec<u32>,
    /// A Vec containing all the Materials set
    pub materials: Vec<Material>,
}
