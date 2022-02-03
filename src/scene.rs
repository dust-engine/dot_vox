// For some reason, the parser combinator definitions in this file won't compile
// without the following while in other files they work just fine:
#![allow(missing_docs)]

use crate::{
    parser::{le_i32, le_u32, parse_dict},
    Dict,
};

use nom::types::CompleteByteSlice;

lazy_static! {
    pub static ref DEFAULT_SCENE: Vec<SceneNode> = vec![
        SceneNode::Transform {
            attributes: Dict::new(),
            frames: vec![Dict::new()],
            child: 1
        },
        SceneNode::Group {
            attributes: Dict::new(),
            children: vec![2]
        },
        SceneNode::Transform {
            attributes: Dict::new(),
            frames: vec![Dict::from([("_t".to_string(), "0 0 1".to_string())])],
            child: 3,
        },
        SceneNode::Shape {
            attributes: Dict::new(),
            models: vec![ShapeModel {
                model_id: 0,
                attributes: Dict::new()
            }]
        }
    ];
}

lazy_static! {
    pub static ref DEFAULT_LAYERS: Vec<Dict> = vec![
        Dict::from([("_name".to_string(), "0".to_string())]),
        Dict::from([("_name".to_string(), "1".to_string())]),
        Dict::from([("_name".to_string(), "2".to_string())]),
        Dict::from([("_name".to_string(), "3".to_string())]),
        Dict::from([("_name".to_string(), "4".to_string())]),
        Dict::from([("_name".to_string(), "5".to_string())]),
        Dict::from([("_name".to_string(), "6".to_string())]),
        Dict::from([("_name".to_string(), "7".to_string())]),
    ];
}

/// Node header.
#[derive(Debug, PartialEq)]
pub struct NodeHeader {
    /// ID of this transform node.
    pub id: u32,
    /// Attributes of this transform node.
    pub attributes: Dict,
}

/// A model reference in a shape node.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeModel {
    /// ID of the model.
    pub model_id: u32,
    /// Attributes of the model in this shape node.
    pub attributes: Dict,
}

/// Transform node.
#[derive(Debug, PartialEq)]
pub struct SceneTransform {
    /// Header.
    pub header: NodeHeader,
    /// 1 single child (appears to always be either a group- or shape node).
    pub child: u32,
    /// Layer ID.
    pub layer_id: i32,
    /// Positional Frames.
    pub frames: Vec<Dict>,
}

/// Group node.
#[derive(Debug, PartialEq)]
pub struct SceneGroup {
    /// Header.
    pub header: NodeHeader,
    /// Multiple children (appear to always be transform nodes).
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
    /// ID of this layer.
    pub id: u32,
    /// Attributes of this layer.
    pub attributes: Dict,
}

named!(parse_node_header <CompleteByteSlice, NodeHeader>, do_parse!(
    id: le_u32 >>
    attributes: parse_dict >>
    (NodeHeader{id, attributes})
));

named!(parse_scene_shape_model <CompleteByteSlice, ShapeModel>, do_parse!(
    model_id: le_u32 >>
    attributes: parse_dict >>
    (ShapeModel{model_id, attributes})
));

named!(pub parse_scene_transform <CompleteByteSlice, SceneTransform>, do_parse!(
    header: parse_node_header >>
    child: le_u32 >>
    _ignored: le_i32 >>
    layer_id: le_i32 >>
    frame_count: le_u32 >>
    frames: many_m_n!(frame_count as usize, frame_count as usize, parse_dict) >>
    (SceneTransform{header, child, layer_id, frames})
));

named!(pub parse_scene_group <CompleteByteSlice, SceneGroup>, do_parse!(
    header: parse_node_header >>
    child_count: le_u32 >>
    children: many_m_n!(child_count as usize, child_count as usize, le_u32) >>
    (SceneGroup{header, children})
));

named!(pub parse_scene_shape <CompleteByteSlice, SceneShape>, do_parse!(
    header: parse_node_header >>
    model_count: le_u32 >>
    models: many_m_n!(model_count as usize, model_count as usize, parse_scene_shape_model) >>
    (SceneShape{header, models})
));

named!(pub parse_layer <CompleteByteSlice, Layer>, do_parse!(
    id: le_u32 >>
    attributes: parse_dict >>
    _ignored: le_i32 >>
    (Layer{id, attributes})
));

/// Scene graph nodes for representing a scene in DotVoxData.
#[derive(Clone, Debug, PartialEq)]
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
