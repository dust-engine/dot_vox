use dot_vox::{DotVoxData, Model, SceneNode};
use nalgebra::{Matrix3, Vector3};

/// Converts the given byte value to a rotation matrix
/// Rotation matrix in voxel context enables 90 degr rotations only, so the contents of the matrix is restricted to 0,1,-1
/// Takes into consideration, that the stored matrix is row-major, while Matrix3 storage is column major
fn parse_rotation_matrix(b: u8) -> Matrix3<i8> {
    let mut result = Matrix3::<i8>::new(0, 0, 0, 0, 0, 0, 0, 0, 0);

    // decide absolute values of each row
    let index_in_first_row = b & 0x3;
    let index_in_second_row = ((b & (0x3 << 2)) >> 2) & 0x3;
    let index_in_third_row = !(index_in_first_row ^ index_in_second_row) & 0x3;
    debug_assert!(index_in_first_row < 3);
    debug_assert!(index_in_second_row < 3);
    debug_assert!(index_in_third_row < 3);
    debug_assert!(index_in_first_row != index_in_second_row);
    debug_assert!(index_in_first_row != index_in_third_row);
    debug_assert!(index_in_second_row != index_in_third_row);

    // decide the sign of the values
    let sign_first_row = if 0 == (b & 0x10) { 1 } else { -1 };
    let sign_second_row = if 0 == (b & 0x20) { 1 } else { -1 };
    let sign_third_row = if 0 == (b & 0x40) { 1 } else { -1 };

    // set the values in the matrix
    result.data.0[index_in_first_row as usize][0] = sign_first_row;
    result.data.0[index_in_second_row as usize][1] = sign_second_row;
    result.data.0[index_in_third_row as usize][2] = sign_third_row;

    result
}

fn iterate_vox_tree<F: FnMut(&Model, &Vector3<i32>, &Matrix3<i8>) -> ()>(
    vox_tree: &DotVoxData,
    mut fun: F,
) {
    // An iterator for each level of depth, during depth-first search:
    // 0 : index inside scene vector
    // 1 : node global translation
    // 2 : node global rotation
    // 3 : helper variable to store iteration state (i.e. to decide the direction of the iteration)
    let mut node_stack: Vec<(u32, Vector3<i32>, Matrix3<i8>, u32)> = Vec::new();

    match &vox_tree.scenes[0] {
        SceneNode::Transform {
            attributes: _,
            frames: _,
            child,
            layer_id: _,
        } => {
            node_stack.push((*child, Vector3::new(0, 0, 0), Matrix3::identity(), 0));
        }
        _ => {
            panic!("The root node for a magicka voxel DAG should be a Transform node")
        }
    }

    while 0 < node_stack.len() {
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
                        + Vector3::new(
                            translation_delta[0],
                            translation_delta[1],
                            translation_delta[2],
                        )
                } else {
                    translation
                };
                let orientation = if let Some(r) = frames[0].attributes.get("_r") {
                    rotation
                        * parse_rotation_matrix(
                            r.parse()
                                .expect("Expected valid u8 byte to parse rotation matrix"),
                        )
                } else {
                    rotation
                };

                // the index variable for a Transform stores whether to go above or below a level next
                if 0 == index {
                    // 0 == index ==> iterate into the child of the translation
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
                node_stack.pop();
                if let Some(parent) = node_stack.last_mut() {
                    parent.3 += 1;
                }
            }
        }
    }
}

fn transform(vec: Vector3<i32>, matrix: &Matrix3<i8>) -> Vector3<i32> {
    Vector3::new(
        vec.x * matrix.m11 as i32 + vec.y * matrix.m12 as i32 + vec.z * matrix.m13 as i32,
        vec.x * matrix.m21 as i32 + vec.y * matrix.m22 as i32 + vec.z * matrix.m23 as i32,
        vec.x * matrix.m31 as i32 + vec.y * matrix.m32 as i32 + vec.z * matrix.m33 as i32,
    )
}

fn main() {
    let vox_tree = dot_vox::load("src/resources/axes.vox")
        .ok()
        .expect("Expected a valid vox file");

    iterate_vox_tree(&vox_tree, |model, position, orientation| {
        let model_size = transform(
            //conversion to Vector3<i32> is required, because orientation might negate the sign of the size components
            Vector3::new(
                model.size.x as i32,
                model.size.y as i32,
                model.size.z as i32,
            ),
            orientation,
        );

        // The global position points to the middle of the model, the element at [0][0][0] is at the bottom left corner
        println!(
            "model size: {model_size} position of element[0][0][0]: {} =========================================================",
            position - (model_size / 2)
        );
    });
}
