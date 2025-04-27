use dot_vox::{DotVoxData, Model, Rotation, SceneNode};
use glam::Vec3;

fn iterate_vox_tree(vox_tree: &DotVoxData, mut fun: impl FnMut(&Model, &Vec3, &Rotation)) {
    match &vox_tree.scenes[0] {
        SceneNode::Transform {
            attributes: _,
            frames: _,
            child,
            layer_id: _,
        } => {
            iterate_vox_tree_inner(
                vox_tree,
                *child,
                Vec3::new(0.0, 0.0, 0.0),
                Rotation::IDENTITY,
                &mut fun,
            );
        }
        _ => {
            panic!("The root node for a magicka voxel DAG should be a Transform node")
        }
    }
}

fn iterate_vox_tree_inner(
    vox_tree: &DotVoxData,
    current_node: u32,
    translation: Vec3,
    rotation: Rotation,
    fun: &mut impl FnMut(&Model, &Vec3, &Rotation),
) {
    match &vox_tree.scenes[current_node as usize] {
        SceneNode::Transform {
            attributes: _,
            frames,
            child,
            layer_id: _,
        } => {
            // In case of a Transform node, the potential translation and rotation is added
            // to the global transform to all of the nodes children nodes
            let translation = if let Some(t) = frames[0].attributes.get("_t") {
                let translation_delta = t
                    .split(" ")
                    .map(|x| x.parse().expect("Not an integer!"))
                    .collect::<Vec<i32>>();
                debug_assert_eq!(translation_delta.len(), 3);
                translation
                    + Vec3::new(
                        translation_delta[0] as f32,
                        translation_delta[1] as f32,
                        translation_delta[2] as f32,
                    )
            } else {
                translation
            };
            let rotation = if let Some(r) = frames[0].attributes.get("_r") {
                rotation
                    * Rotation::from_byte(
                        r.parse()
                            .expect("Expected valid u8 byte to parse rotation matrix"),
                    )
            } else {
                Rotation::IDENTITY
            };

            iterate_vox_tree_inner(vox_tree, *child, translation, rotation, fun);
        }
        SceneNode::Group {
            attributes: _,
            children,
        } => {
            // in case the current node is a group, the index variable stores the current
            // child index
            for child_node in children {
                iterate_vox_tree_inner(vox_tree, *child_node, translation, rotation, fun);
            }
        }
        SceneNode::Shape {
            attributes: _,
            models,
        } => {
            // in case the current node is a shape: it's a leaf node and it contains
            // models(voxel arrays)
            for model in models {
                fun(
                    &vox_tree.models[model.model_id as usize],
                    &translation,
                    &rotation,
                );
            }
        }
    }
}

fn main() {
    let vox_tree = dot_vox::load("src/resources/axes.vox")
        .ok()
        .expect("Expected a valid vox file");

    iterate_vox_tree(&vox_tree, |model, position, orientation| {
        //conversion to Vec3<i32> is required, because orientation might negate the
        // sign of the size components
        let model_size = glam::Mat3::from_cols_array_2d(&orientation.to_cols_array_2d())
            * Vec3::new(
                model.size.x as f32,
                model.size.y as f32,
                model.size.z as f32,
            );

        // The global position points to the middle of the model, the element at
        // [0][0][0] is at the bottom left corner
        println!(
            "model size: {model_size} position of element[0][0][0]: {}",
            *position - (model_size / 2.)
        );
    });
}
