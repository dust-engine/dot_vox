use dot_vox::{DotVoxData, Model, Rotation, SceneNode};
use glam::Vec3;

fn iterate_vox_tree(vox_tree: &DotVoxData, mut fun: impl FnMut(&Model, &Vec3, &Rotation)) {
    // The stack for maintaining iteration state.
    // 0 : index inside scene vector
    // 1 : node global translation
    // 2 : node global rotation
    // 3 : helper variable to store iteration state (i.e. to decide the direction of the iteration)
    let mut node_stack: Vec<(u32, Vec3, Rotation, u32)> = Vec::new();

    match &vox_tree.scenes[0] {
        SceneNode::Transform {
            attributes: _,
            frames: _,
            child,
            layer_id: _,
        } => {
            node_stack.push((*child, Vec3::new(0.0, 0.0, 0.0), Rotation::IDENTITY, 0));
        }
        _ => {
            panic!("The root node for a magicka voxel DAG should be a Transform node")
        }
    }

    while !node_stack.is_empty() {
        let (current_node, translation, rotation, index) = *node_stack.last().unwrap();
        match &vox_tree.scenes[current_node as usize] {
            SceneNode::Transform {
                attributes: _,
                frames,
                child,
                layer_id: _,
            } => {
                // In case of a Transform node, the potential translation and rotation is added to the global transform to all of the nodes children nodes
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
                let orientation = if let Some(r) = frames[0].attributes.get("_r") {
                    rotation
                        * Rotation::from_byte(
                            r.parse()
                                .expect("Expected valid u8 byte to parse rotation matrix"),
                        )
                } else {
                    rotation
                };

                // the index variable for a Transform stores whether to go above or below a level next
                if 0 == index {
                    // 0 == index ==> iterate into the child of the translation
                    // Also update index value to mark this stack item to be removed the next time it is encountered,
                    // Which is going to be after the algo is done with the child nodes of this node
                    node_stack.last_mut().unwrap().3 += 1;
                    node_stack.push((*child, translation, orientation, 0));
                } else {
                    // 0 != index ==> remove translation and iterate into parent
                    node_stack.pop();
                }
            }
            SceneNode::Group {
                attributes: _,
                children,
            } => {
                // in case the current node is a group, the index variable stores the current child index
                if (index as usize) < children.len() {
                    node_stack.last_mut().unwrap().3 += 1;
                    node_stack.push((children[index as usize], translation, rotation, 0));
                } else {
                    node_stack.pop();
                }
            }
            SceneNode::Shape {
                attributes: _,
                models,
            } => {
                // in case the current node is a shape: it's a leaf node and it contains models(voxel arrays)
                for model in models {
                    fun(
                        &vox_tree.models[model.model_id as usize],
                        &translation,
                        &rotation,
                    );
                }
                node_stack.pop(); // stack popped to the parent
                if let Some(parent) = node_stack.last_mut() {
                    // Update the index of the parent group to go to the next child
                    parent.3 += 1;
                }
            }
        }
    }
}

fn main() {
    let vox_tree = dot_vox::load("src/resources/axes.vox")
        .ok()
        .expect("Expected a valid vox file");

    iterate_vox_tree(&vox_tree, |model, position, orientation| {
        //conversion to Vec3<i32> is required, because orientation might negate the sign of the size components
        let model_size = glam::Mat3::from_cols_array_2d(&orientation.to_cols_array_2d())
            * Vec3::new(
                model.size.x as f32,
                model.size.y as f32,
                model.size.z as f32,
            );

        // The global position points to the middle of the model, the element at [0][0][0] is at the bottom left corner
        println!(
            "model size: {model_size} position of element[0][0][0]: {}",
            *position - (model_size / 2.)
        );
    });
}
