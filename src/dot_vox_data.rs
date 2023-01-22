use crate::{Color, Dict, Layer, Material, Model, SceneNode};
use std::io::{self, Write};

/// Container for `.vox` file data.
#[derive(Debug, PartialEq, Eq)]
pub struct DotVoxData {
    /// The version number of the `.vox` file.
    pub version: u32,
    /// A `Vec` of all the models contained within this file.
    pub models: Vec<Model>,
    /// A `Vec` containing the colour palette as 32-bit integers
    pub palette: Vec<Color>,
    /// A `Vec` containing all the [`Material`]s set.
    pub materials: Vec<Material>,
    /// Scene. The first node in this list is always the root node.
    pub scenes: Vec<SceneNode>,
    /// Layers. Used by scene transform nodes.
    pub layers: Vec<Layer>,
}

impl DotVoxData {
    /// Serializes `self` in the `.vox` format.
    /// TODO: write the material set
    pub fn write_vox<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        self.write_header(writer)?;

        // Write out all of the children of MAIN first to get the number of bytes.
        let mut children_buffer = Vec::new();
        self.write_models(&mut children_buffer)?;
        self.write_scene_graph(&mut children_buffer)?;
        self.write_palette_chunk(&mut children_buffer)?;
        let num_main_children_bytes = children_buffer.len() as u32;

        self.write_main_chunk(writer, num_main_children_bytes)?;

        writer.write_all(&children_buffer)
    }

    fn write_header<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_all("VOX ".as_bytes())?;
        writer.write_all(&self.version.to_le_bytes())
    }

    fn write_main_chunk<W: Write>(
        &self,
        writer: &mut W,
        num_children_bytes: u32,
    ) -> Result<(), io::Error> {
        Self::write_chunk(writer, "MAIN", &[], num_children_bytes)
    }

    fn write_models<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        if self.models.len() > 1 {
            self.write_pack_chunk(writer)?;
        }
        for model in self.models.iter() {
            Self::write_model(writer, model)?;
        }

        Ok(())
    }

    fn write_pack_chunk<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        let chunk = (self.models.len() as u32).to_le_bytes();

        let mut num_children_bytes = 0;
        for model in self.models.iter() {
            num_children_bytes += model.num_vox_bytes();
        }

        Self::write_chunk(writer, "PACK", &chunk, num_children_bytes)
    }

    fn write_model<W: Write>(writer: &mut W, model: &Model) -> Result<(), io::Error> {
        let mut size_chunk = Vec::new();
        size_chunk.extend_from_slice(&model.size.x.to_le_bytes());
        size_chunk.extend_from_slice(&model.size.y.to_le_bytes());
        size_chunk.extend_from_slice(&model.size.z.to_le_bytes());
        Self::write_leaf_chunk(writer, "SIZE", &size_chunk)?;

        let mut xyzi_chunk = Vec::new();
        xyzi_chunk.extend_from_slice(&(model.voxels.len() as u32).to_le_bytes());
        for voxel in model.voxels.iter() {
            xyzi_chunk.push(voxel.x);
            xyzi_chunk.push(voxel.y);
            xyzi_chunk.push(voxel.z);
            // `Voxel::i` uses 0-based palette indices, while VOX uses 1-based.
            xyzi_chunk.push(voxel.i + 1);
        }
        Self::write_leaf_chunk(writer, "XYZI", &xyzi_chunk)
    }

    fn write_string(buffer: &mut Vec<u8>, str: &String) {
        buffer.extend_from_slice(&((str.len() as u32).to_le_bytes()));
        buffer.extend_from_slice(&str.as_bytes());
    }

    fn write_dict(buffer: &mut Vec<u8>, dict: &Dict) {
        buffer.extend_from_slice(&((dict.len() as u32).to_le_bytes()));
        for (key, value) in dict.iter() {
            Self::write_string(buffer, key);
            Self::write_string(buffer, value);
        }
    }

    fn write_scene_graph<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        for (i, node) in self.scenes.iter().enumerate() {
            Self::write_scene_node(writer, node, i as u32)?;
        }

        Ok(())
    }

    fn write_scene_node<W: Write>(
        writer: &mut W,
        node: &SceneNode,
        i: u32,
    ) -> Result<(), io::Error> {
        let id;
        let mut node_chunk = Vec::new();
        match node {
            SceneNode::Group {
                attributes,
                children,
            } => {
                id = "nGRP";
                node_chunk.extend_from_slice(&(i as u32).to_le_bytes());
                Self::write_dict(&mut node_chunk, &attributes);
                node_chunk.extend_from_slice(&((children.len() as u32).to_le_bytes()));
                for child in children {
                    node_chunk.extend_from_slice(&child.to_le_bytes());
                }
            }
            SceneNode::Transform {
                frames,
                child,
                layer_id,
                attributes,
            } => {
                id = "nTRN";
                node_chunk.extend_from_slice(&(i as u32).to_le_bytes());
                Self::write_dict(&mut node_chunk, &attributes);
                node_chunk.extend_from_slice(&child.to_le_bytes());
                node_chunk.extend_from_slice(&u32::MAX.to_le_bytes());
                node_chunk.extend_from_slice(&layer_id.to_le_bytes());
                node_chunk.extend_from_slice(&(frames.len() as u32).to_le_bytes());
                for frame in frames {
                    Self::write_dict(&mut node_chunk, &frame.attributes);
                }
            }
            SceneNode::Shape { attributes, models } => {
                id = "nSHP";
                node_chunk.extend_from_slice(&(i as u32).to_le_bytes());
                Self::write_dict(&mut node_chunk, &attributes);
                node_chunk.extend_from_slice(&(models.len() as u32).to_le_bytes());
                for model in models {
                    node_chunk.extend_from_slice(&model.model_id.to_le_bytes());
                    Self::write_dict(&mut node_chunk, &model.attributes);
                }
            }
        }

        Self::write_leaf_chunk(writer, id, &node_chunk)
    }

    fn write_palette_chunk<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        let mut chunk = Vec::new();
        for color in self.palette.iter() {
            let color: [u8; 4] = color.into();
            chunk.extend_from_slice(&color);
        }

        Self::write_leaf_chunk(writer, "RGBA", &chunk)
    }

    fn write_leaf_chunk<W: Write>(writer: &mut W, id: &str, chunk: &[u8]) -> Result<(), io::Error> {
        let num_children_bytes: u32 = 0;

        Self::write_chunk(writer, id, chunk, num_children_bytes)
    }

    fn write_chunk<W: Write>(
        writer: &mut W,
        id: &str,
        chunk: &[u8],
        num_children_bytes: u32,
    ) -> Result<(), io::Error> {
        assert!(id.len() == 4);
        writer.write_all(id.as_bytes())?;
        writer.write_all(&(chunk.len() as u32).to_le_bytes())?;
        writer.write_all(&num_children_bytes.to_le_bytes())?;
        writer.write_all(chunk)
    }
}
