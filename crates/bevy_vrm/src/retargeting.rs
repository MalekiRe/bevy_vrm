use crate::humanoid_bones::HumanoidBonesInitialized;
use crate::loader::Vrm;
use crate::HumanoidBones;
use bevy::math::Affine3A;
use bevy::math::EulerRot::XYZ;
use bevy::prelude::*;
use bevy::render::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::scene::SceneInstance;
use bevy::transform;
use bevy::utils::HashMap;
use bevy_gltf_kun::import::gltf::node::GltfNode;
use serde_vrm::vrm0::BoneName;
use std::f32::consts::PI;
use std::ops::{Deref, DerefMut};

pub struct VrmRetargetingPlugin;

impl Plugin for VrmRetargetingPlugin {
    fn build(&self, app: &mut App) {
        //app.add_systems(Update, rotate_scene);
        app.add_systems(Update, retarget_vrm);
    }
}

#[derive(Component)]
pub struct VrmRetargetingInitialized;

#[derive(Component)]
struct Flipped;
pub fn rotate_scene(
    mut commands: Commands,
    mut nodes: Query<(Entity, &mut Transform), (With<Handle<GltfNode>>, Without<Flipped>)>,
) {
    let transform_180 = Transform::from_rotation(Quat::from_rotation_x(PI));
    for (e, mut node) in nodes.iter_mut() {
        commands.entity(e).insert(Flipped);
        let rest = transform_180 * *node * transform_180;
        node.rotation = transform_180.rotation * rest.rotation * transform_180.rotation;
        node.translation = rest.translation;
        *node = transform_180 * *node * transform_180;
    }
}

pub fn retarget_vrm(
    mut local: Local<bool>,
    mut commands: Commands,
    mut vrm: Query<
        (Entity, &HumanoidBones),
        (
            Without<VrmRetargetingInitialized>,
            With<HumanoidBonesInitialized>,
        ),
    >,
    skinned_meshes: Query<&SkinnedMesh>,
    mut skinned_mesh_inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    children: Query<&Children>,
    parents: Query<&Parent>,
    mut local_transforms: Query<&mut Transform>,
    global_transforms: Query<&GlobalTransform>,
) {
    for (entity, humanoid_bones) in vrm.iter_mut() {
        if *local == true {
            return;
        }
        *local = true;
        let left_shoulder = *humanoid_bones.0.get(&BoneName::LeftShoulder).unwrap();
        let left_upper_arm = *humanoid_bones.0.get(&BoneName::LeftUpperArm).unwrap();
        let left_lower_arm = *humanoid_bones.0.get(&BoneName::LeftLowerArm).unwrap();
        let left_hand = *humanoid_bones.0.get(&BoneName::LeftHand).unwrap();

        let shoulder_transform = Transform::from_rotation(Quat::from_xyzw(0.5, -0.5, -0.5, -0.5));
        let upper_arm_transform = Transform::from_rotation(Quat::from_xyzw(0.0, 1.0, 0.0, 0.0));
        let lower_arm_transform = Transform::from_rotation(Quat::from_xyzw(0.0, 0.707, 0.0, 0.707));
        let hand_transform = Transform::from_rotation(Quat::from_xyzw(0.0, -0.708, 0.0, 0.707));
        retarget_entity(
            shoulder_transform.rotation,
            left_shoulder,
            &skinned_meshes,
            &mut skinned_mesh_inverse_bindposes,
            &children,
            &mut local_transforms,
        );
        retarget_entity(
            upper_arm_transform.rotation,
            left_upper_arm,
            &skinned_meshes,
            &mut skinned_mesh_inverse_bindposes,
            &children,
            &mut local_transforms,
        );
        retarget_entity(
            lower_arm_transform.rotation,
            left_lower_arm,
            &skinned_meshes,
            &mut skinned_mesh_inverse_bindposes,
            &children,
            &mut local_transforms,
        );
        retarget_entity(
            hand_transform.rotation,
            left_hand,
            &skinned_meshes,
            &mut skinned_mesh_inverse_bindposes,
            &children,
            &mut local_transforms,
        );
    }
}

fn retarget_entity(
    // this rotation should be in local space
    new_rot: Quat,
    entity: Entity,
    skinned_meshes: &Query<&SkinnedMesh>,
    mut skinned_mesh_inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    children: &Query<&Children>,
    mut local_transforms: &mut Query<&mut Transform>,
) {
    // setup
    let mut this_transform = local_transforms.get_mut(entity).unwrap();
    let old_rot = this_transform.rotation;
    let old_bind =
        get_skinned_mesh(entity, &skinned_meshes, &mut skinned_mesh_inverse_bindposes).unwrap();
    // this should be the rotation that maps the old rotation to the new rotation
    let comp_rot = (old_rot.inverse() * new_rot).normalize();
    // set the bindpose
    set_skinned_mesh(
        entity,
        old_bind.rotation * comp_rot,
        &skinned_meshes,
        &mut skinned_mesh_inverse_bindposes,
    );
    // set the rotation of the arm
    // this needs to be set to the new rot in local space
    this_transform.rotation = new_rot;
    // correct the positions of all of the child entities
    for child in children.iter_descendants(entity) {
        let that_bind =
            get_skinned_mesh(child, &skinned_meshes, &mut skinned_mesh_inverse_bindposes).unwrap();
        set_skinned_mesh(
            child,
            (that_bind.rotation * comp_rot).normalize(),
            skinned_meshes,
            skinned_mesh_inverse_bindposes,
        );
        // fix translations
        let mut that_transform = local_transforms.get_mut(child).unwrap();
        that_transform.translation = comp_rot.inverse() * that_transform.translation;
    }
}

fn get_skinned_mesh(
    entity: Entity,
    skinned_meshes: &Query<&SkinnedMesh>,
    skinned_mesh_inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
) -> Option<Transform> {
    for skinned_mesh in skinned_meshes.iter() {
        let joints = &skinned_mesh.joints;
        let inverse_bind_pose = skinned_mesh_inverse_bindposes
            .get_mut(&skinned_mesh.inverse_bindposes)
            .unwrap();
        for (i, joint) in joints.iter().enumerate() {
            if *joint == entity {
                let mut temp = inverse_bind_pose.to_vec();
                let t = temp.get(i).unwrap().clone();
                return Some(Transform::from_matrix(t.inverse()));
            }
        }
    }
    None
}

fn set_skinned_mesh(
    entity: Entity,
    new_rot: Quat,
    skinned_meshes: &Query<&SkinnedMesh>,
    skinned_mesh_inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
) {
    for skinned_mesh in skinned_meshes.iter() {
        let joints = &skinned_mesh.joints;
        let inverse_bind_pose = skinned_mesh_inverse_bindposes
            .get_mut(&skinned_mesh.inverse_bindposes)
            .unwrap();
        for (i, joint) in joints.iter().enumerate() {
            if *joint == entity {
                let mut temp = inverse_bind_pose.to_vec();
                let t = temp.get(i).unwrap().clone();
                // we inverse T because it's the inverse bindpose, we inverse delta worldspace because it's
                // what we gotta offset it by?
                let mut temp2 = Transform::from_matrix(t.inverse());
                temp2.rotation = new_rot;
                *temp.get_mut(i).unwrap() = temp2.compute_matrix().inverse();
                *inverse_bind_pose = SkinnedMeshInverseBindposes::from(temp);
                return;
            }
        }
    }
}

fn skeleton_rotate(
    parents: &Query<&Parent>,
    children: &Query<&Children>,
    local_transforms: &mut Query<&mut Transform>,
    global_transforms: &Query<&GlobalTransform>,
    humanoid_bones: &HumanoidBones,
    old_skeleton_global_rest: &mut SkeletonProfileHumanoidGlobal,
    old_skeleton_local_rest: &mut SkeletonProfileHumanoid,
) {
    let mut prof_skeleton = SkeletonProfileHumanoidGlobal::default();

    let mut diffs = HashMap::new();

    for (bone_name, entity) in humanoid_bones.0.iter() {
        let transform = local_transforms.get(*entity).unwrap().clone();
        let global_transform = global_transforms.get(*entity).unwrap().clone();
        old_skeleton_local_rest
            .0
            .insert(bone_name.clone(), transform);
        old_skeleton_global_rest
            .0
            .insert(bone_name.clone(), global_transform.compute_transform());
        diffs.insert(entity.clone(), Quat::IDENTITY);
    }

    // We need to process all the parentless bones,
    // For now we are just assuming that the hip is that one
    // We can figure this out later cause it isn't always true
    let hip_which_is_root: Entity = *humanoid_bones.0.get(&BoneName::Hips).unwrap();

    let mut bones_to_process = vec![hip_which_is_root];

    for bone in bones_to_process.clone().into_iter() {
        bones_to_process.clear();
        for child in children.get(bone).unwrap() {
            bones_to_process.push(*child);
        }

        let src_pg = if let Ok(parent_bone) = parents.get(bone) {
            let parent_bone = parent_bone.get();
            global_transforms
                .get(parent_bone)
                .unwrap()
                .to_scale_rotation_translation()
                .1
        } else {
            Quat::IDENTITY
        };

        let tgt_rot = if let Some(matching_bone_name) = humanoid_bones.0.iter().find_map(|map| {
            if map.1.clone() == bone {
                Some(map.0.clone())
            } else {
                None
            }
        }) {
            src_pg.inverse() * prof_skeleton.0.get(&matching_bone_name).unwrap().rotation
        } else {
            Quat::IDENTITY
        };

        if let Ok(parent_bone) = parents.get(bone) {
            (tgt_rot.inverse()
                * diffs.get(&parent_bone.get()).unwrap().clone()
                * local_transforms.get(bone).unwrap().rotation)
        } else {
            tgt_rot.inverse() * local_transforms.get(bone).unwrap().rotation
        };
    }
}

pub struct SkeletonProfileHumanoid(pub HashMap<BoneName, Transform>);

pub struct SkeletonProfileHumanoidGlobal(pub HashMap<BoneName, Transform>);

impl Default for SkeletonProfileHumanoidGlobal {
    fn default() -> Self {
        let mut skeleton_profile_humanoid = SkeletonProfileHumanoid::default();

        let mut global = HashMap::new();

        for (bone, transform) in skeleton_profile_humanoid.0.iter() {
            let mut current_bone = bone.clone();
            let mut transforms = vec![];
            while let Some(next_bone) = current_bone.parent() {
                transforms.insert(
                    0,
                    skeleton_profile_humanoid.0.get(&next_bone).unwrap().clone(),
                );
            }

            let mut current_transform = Transform::default();
            for parent_transform in transforms {
                current_transform = current_transform.mul_transform(parent_transform);
            }

            global.insert(bone.clone(), current_transform);
        }

        Self(global)
    }
}

impl Default for SkeletonProfileHumanoid {
    fn default() -> Self {
        let mut this = HashMap::new();
        this.insert(
            BoneName::Hips,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.75, 0.0,
            ]))),
        );
        this.insert(
            BoneName::Spine,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.1, 0.0,
            ]))),
        );
        this.insert(
            BoneName::Chest,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.1, 0.0,
            ]))),
        );
        this.insert(
            BoneName::UpperChest,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.1, 0.0,
            ]))),
        );
        this.insert(
            BoneName::Neck,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.1, 0.0,
            ]))),
        );
        this.insert(
            BoneName::Head,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.1, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftEye,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.05, 0.15, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightEye,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 0.0, -0.05, 0.15, 0.0,
            ]))),
        );
        this.insert(
            BoneName::Jaw,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.05, 0.05,
            ]))),
        );
        this.insert(
            BoneName::LeftShoulder,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.05, 0.1, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftUpperArm,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.05, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftLowerArm,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                0.0, 0.0, 1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.25, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftHand,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                0.0, 0.0, -1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.25, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftThumbProximal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                0.0, 0.0, -1.0, -0.577, 0.816, 0.0, 0.816, 0.577, 0.0, -0.025, 0.025, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftThumbIntermediate,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.043, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftThumbDistal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.043, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftIndexProximal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, -0.025, 0.075, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftIndexIntermediate,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.05, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftIndexDistal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.025, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftMiddleProximal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.075, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftMiddleIntermediate,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.075, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftMiddleDistal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.025, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftRingProximal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.025, 0.075, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftRingIntermediate,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.05, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftRingDistal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.025, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftLittleProximal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.05, 0.05, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftLittleIntermediate,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.05, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftLittleDistal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.025, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightShoulder,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                0.0, 0.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, -0.05, 0.1, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightUpperArm,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.05, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightLowerArm,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                0.0, 0.0, -1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.25, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightHand,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                0.0, 0.0, 1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.25, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightThumbProximal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                0.0, 0.0, 1.0, 0.577, 0.816, 0.0, -0.816, 0.577, 0.0, 0.025, 0.025, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightThumbIntermediate,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.043, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightThumbDistal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.043, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightIndexProximal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.025, 0.075, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightIndexIntermediate,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.05, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightIndexDistal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.025, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightMiddleProximal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.075, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightMiddleIntermediate,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.075, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightMiddleDistal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.025, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightRingProximal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, -0.025, 0.075, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightRingIntermediate,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.05, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightRingDistal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.025, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightLittleProximal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, -0.05, 0.05, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightLittleIntermediate,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.05, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightLittleDistal,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.025, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftUpperLeg,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.1, 0.0, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftLowerLeg,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.375, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftFoot,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                -1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, -1.0, 0.0, 0.0, 0.375, 0.0,
            ]))),
        );
        this.insert(
            BoneName::LeftToes,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.15, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightUpperLeg,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0, -0.1, 0.0, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightLowerLeg,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.375, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightFoot,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                -1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, -1.0, 0.0, 0.0, 0.375, 0.0,
            ]))),
        );
        this.insert(
            BoneName::RightToes,
            Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[
                -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.15, 0.0,
            ]))),
        );
        Self(this)
    }
}

impl Deref for SkeletonProfileHumanoid {
    type Target = HashMap<BoneName, Transform>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SkeletonProfileHumanoid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}