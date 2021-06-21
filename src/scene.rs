// For some reason, the parser combinator definitions in this file won't compile without the following while in other files they work just fine:
#![allow(missing_docs)]

use {Dict};
use ::parser::{le_u32, parse_dict};
use nom::types::CompleteByteSlice;

/// Node header.
#[derive(Debug, PartialEq)]
pub struct NodeHeader
{
    /// Id of this transform node
    pub id: u32,
    /// Attributes of this transform node
    pub attributes: Dict,
}

/// A model reference in a shape node.
#[derive(Debug, PartialEq)]
pub struct ShapeNodeModel
{
    /// Id of the model.
    pub model_id: u32,
    /// Attributes of the model in this shape node (reserved; no known meaningful values).
    pub attributes: Dict,
}

/// Transform node.
#[derive(Debug, PartialEq)]
pub struct TransformNode
{
    /// Header
    pub header: NodeHeader,
    /// 1 single child (appear to be always either a group or shape node)
    pub child: u32,
    /// Layer ID.
    pub layer_id: u32,
    /// Positional Frames.
    pub frames: Vec<Dict>,
}

/// Group node.
#[derive(Debug, PartialEq)]
pub struct GroupNode
{
    /// Header
    pub header: NodeHeader,
    /// Multiple children (appear to be always transform nodes)
    pub children: Vec<u32>,
}

/// Shape node.
#[derive(Debug, PartialEq)]
pub struct ShapeNode
{
    /// Header
    pub header: NodeHeader,
    /// 1 or more models
    pub models: Vec<ShapeNodeModel>,
}

/// Layer information.
#[derive(Debug, PartialEq)]
pub struct Layer
{
    /// id of this layer.
    id: u32,
    /// Attributes of this layer
    attributes: Dict,
}

named!(parse_node_header <CompleteByteSlice, NodeHeader>, do_parse!(
    id: le_u32 >>
    attributes: parse_dict >>
    (NodeHeader{id, attributes})
));

named!(parse_scene_shape_model <CompleteByteSlice, ShapeNodeModel>, do_parse!(
    model_id: le_u32 >>
    attributes: parse_dict >>
    (ShapeNodeModel{model_id, attributes})
));

named!(pub parse_scene_transform <CompleteByteSlice, TransformNode>, do_parse!(
    header: parse_node_header >>
    child: le_u32 >>
    _ignored: le_u32 >>
    layer_id: le_u32 >>
    frame_count: le_u32 >>
    frames: many_m_n!(frame_count as usize, frame_count as usize, parse_dict) >>
    (TransformNode{header, child, layer_id, frames})
));

named!(pub parse_scene_group <CompleteByteSlice, GroupNode>, do_parse!(
    header: parse_node_header >>
    child_count: le_u32 >>
    children: many_m_n!(child_count as usize, child_count as usize, le_u32) >>
    (GroupNode{header, children})
));

named!(pub parse_scene_shape <CompleteByteSlice, ShapeNode>, do_parse!(
    header: parse_node_header >>
    model_count: le_u32 >>
    models: many_m_n!(model_count as usize, model_count as usize, parse_scene_shape_model) >>
    (ShapeNode{header, models})
));

named!(pub parse_layer <CompleteByteSlice, Layer>, do_parse!(
    id: le_u32 >>
    attributes: parse_dict >>
    _ignored: le_u32 >>
    (Layer{id, attributes})
));
