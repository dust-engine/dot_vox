// For some reason, the parser combinator definitions in this file won't compile without the following while in other files they work just fine:
#![allow(missing_docs)]

use nom::number::complete::{le_i32, le_u32};
use nom::{multi::count, sequence::pair, IResult};
use crate::parser::parse_dict;
use crate::Dict;

/// Node header.
#[derive(Debug, PartialEq)]
pub struct NodeHeader {
    /// Id of this transform node
    pub id: u32,
    /// Attributes of this transform node
    pub attributes: Dict,
}

/// A model reference in a shape node.
#[derive(Debug, PartialEq)]
pub struct ShapeModel {
    /// Id of the model.
    pub model_id: u32,
    /// Attributes of the model in this shape node.
    pub attributes: Dict,
}

/// Transform node.
#[derive(Debug, PartialEq)]
pub struct SceneTransform {
    /// Header
    pub header: NodeHeader,
    /// 1 single child (appear to be always either a group or shape node)
    pub child: u32,
    /// Layer ID.
    pub layer_id: i32,
    /// Positional Frames.
    pub frames: Vec<Dict>,
}

/// Group node.
#[derive(Debug, PartialEq)]
pub struct SceneGroup {
    /// Header
    pub header: NodeHeader,
    /// Multiple children (appear to be always transform nodes)
    pub children: Vec<u32>,
}

/// Shape node.
#[derive(Debug, PartialEq)]
pub struct SceneShape {
    /// Header
    pub header: NodeHeader,
    /// 1 or more models
    pub models: Vec<ShapeModel>,
}

/// Layer information.
#[derive(Debug, PartialEq)]
pub struct Layer {
    /// id of this layer.
    pub id: u32,
    /// Attributes of this layer
    pub attributes: Dict,
}

fn parse_node_header(i: &[u8]) -> IResult<&[u8], NodeHeader> {
    let (i, (id, attributes)) = pair(le_u32, parse_dict)(i)?;
    Ok((i, NodeHeader { id, attributes }))
}

fn parse_scene_shape_model(i: &[u8]) -> IResult<&[u8], ShapeModel> {
    let (i, (model_id, attributes)) = pair(le_u32, parse_dict)(i)?;
    Ok((
        i,
        ShapeModel {
            model_id,
            attributes,
        },
    ))
}

pub fn parse_scene_transform(i: &[u8]) -> IResult<&[u8], SceneTransform> {
    let (i, header) = parse_node_header(i)?;
    let (i, child) = le_u32(i)?;
    let (i, _ignored) = le_u32(i)?;
    let (i, layer_id) = le_i32(i)?;
    let (i, frame_count) = le_u32(i)?;
    let (i, frames) = count(parse_dict, frame_count as usize)(i)?;
    Ok((
        i,
        SceneTransform {
            header,
            child,
            layer_id,
            frames,
        },
    ))
}

pub fn parse_scene_group(i: &[u8]) -> IResult<&[u8], SceneGroup> {
    let (i, header) = parse_node_header(i)?;
    let (i, child_count) = le_u32(i)?;
    let (i, children) = count(le_u32, child_count as usize)(i)?;
    Ok((i, SceneGroup { header, children }))
}

pub fn parse_scene_shape(i: &[u8]) -> IResult<&[u8], SceneShape> {
    let (i, header) = parse_node_header(i)?;
    let (i, model_count) = le_u32(i)?;
    let (i, models) = count(parse_scene_shape_model, model_count as usize)(i)?;
    Ok((i, SceneShape { header, models }))
}

pub fn parse_layer(i: &[u8]) -> IResult<&[u8], Layer> {
    let (i, id) = le_u32(i)?;
    let (i, attributes) = parse_dict(i)?;
    let (i, _ignored) = le_u32(i)?;
    Ok((i, Layer { id, attributes }))
}

/// Scene graph nodes for representing a scene in DotVoxData.
#[derive(Debug, PartialEq)]
pub enum SceneNode {
    Transform {
        /// Attributes.
        attributes: Dict,
        /// Transform frames.
        frames: Vec<Dict>,
        /// Child node of this Transform node.
        child: u32,
    },
    Group {
        /// Attributes.
        attributes: Dict,
        /// Child nodes
        children: Vec<u32>,
    },
    Shape {
        /// Attributes.
        attributes: Dict,
        /// Models.
        models: Vec<ShapeModel>,
    },
}
