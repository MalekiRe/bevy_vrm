use std::ops::{Deref, DerefMut};
use bevy::math::Affine3A;
use bevy::prelude::*;
use bevy::scene::SceneInstance;
use bevy::utils::HashMap;
use serde_vrm::vrm0::BoneName;
use crate::humanoid_bones::HumanoidBonesInitialized;
use crate::HumanoidBones;
use crate::loader::Vrm;

pub struct VrmRetargetingPlugin;

impl Plugin for VrmRetargetingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, retarget_vrm);
    }
}

#[derive(Component)]
pub struct VrmRetargetingInitialized;

pub fn retarget_vrm(
    mut commands: Commands,
    mut vrm: Query<
        (Entity, &HumanoidBones, &Handle<Vrm>, &SceneInstance),
        (Without<VrmRetargetingInitialized>, With<HumanoidBonesInitialized>),
    >,
    mut local_transforms: Query<&mut Transform>,
    global_transforms: Query<&GlobalTransform>,
    names: Query<(Entity, &Name)>,
    scene_manager: Res<SceneSpawner>,
    vrms: Res<Assets<Vrm>>,
) {
    for (entity, humanoid_bones, handle, instance) in vrm.iter_mut() {
        if !scene_manager.instance_is_ready(**instance) {
            continue;
        }

        //TODO apply_node_transforms, is useful for importing FBX and some rare VRM models https://github.com/V-Sekai/godot-vrm/blob/ef7f8f94c7045dd55d5e93f9c6e7fdfc49500c1e/addons/vrm/vrm_utils.gd#L102

        let mut old_skeleton_global_rest = SkeletonProfileHumanoid::default();
        let mut old_skeleton_local_rest = SkeletonProfileHumanoid::default();

        skeleton_rotate(&mut local_transforms, &global_transforms, humanoid_bones, &mut old_skeleton_global_rest, &mut old_skeleton_local_rest);;


        commands.entity(entity)
            .insert(VrmRetargetingInitialized);
    }
}

fn skeleton_rotate(local_transforms: &mut Query<&mut Transform>, global_transforms: &Query<&GlobalTransform>, humanoid_bones: &HumanoidBones, old_skeleton_global_rest: &mut SkeletonProfileHumanoid, old_skeleton_local_rest: &mut SkeletonProfileHumanoid) {
    let mut prof_skeleton = SkeletonProfileHumanoid::default();

    for (bone_name, entity) in humanoid_bones.0.iter() {
        let transform = local_transforms.get(*entity).unwrap().clone();
        let global_transform = global_transforms.get(*entity).unwrap().clone();
        old_skeleton_local_rest.0.insert(bone_name.clone(), transform);
        old_skeleton_global_rest.0.insert(bone_name.clone(), global_transform.compute_transform());
    }

    // We need to process all the parentless bones,
    // the only parentless bone here should be the Hip Bone
    // In godot this is the root node, but we don't have that in bevy.

    let hip_which_is_root: Entity = *humanoid_bones.0.get(&BoneName::Hips).unwrap();








}



pub struct SkeletonProfileHumanoid(pub HashMap<BoneName, Transform>);


impl Default for SkeletonProfileHumanoid {
    fn default() -> Self {
        let mut this = HashMap::new();
        this.insert(BoneName::Hips, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.75, 0.0]))));
        this.insert(BoneName::Spine, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.1, 0.0]))));
        this.insert(BoneName::Chest, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.1, 0.0]))));
        this.insert(BoneName::UpperChest, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.1, 0.0]))));
        this.insert(BoneName::Neck, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.1, 0.0]))));
        this.insert(BoneName::Head, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.1, 0.0]))));
        this.insert(BoneName::LeftEye, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.05, 0.15, 0.0]))));
        this.insert(BoneName::RightEye, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 0.0, -0.05, 0.15, 0.0]))));
        this.insert(BoneName::Jaw, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[-1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.05, 0.05]))));
        this.insert(BoneName::LeftShoulder, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.05, 0.1, 0.0]))));
        this.insert(BoneName::LeftUpperArm, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[-1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.05, 0.0]))));
        this.insert(BoneName::LeftLowerArm, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[0.0, 0.0, 1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.25, 0.0]))));
        this.insert(BoneName::LeftHand, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[0.0, 0.0, -1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.25, 0.0]))));
        this.insert(BoneName::LeftThumbProximal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[0.0, 0.0, -1.0, -0.577, 0.816, 0.0, 0.816, 0.577, 0.0, -0.025, 0.025, 0.0]))));
        this.insert(BoneName::LeftThumbIntermediate, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.043, 0.0]))));
        this.insert(BoneName::LeftThumbDistal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.043, 0.0]))));
        this.insert(BoneName::LeftIndexProximal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, -0.025, 0.075, 0.0]))));
        this.insert(BoneName::LeftIndexIntermediate, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.05, 0.0]))));
        this.insert(BoneName::LeftIndexDistal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.025, 0.0]))));
        this.insert(BoneName::LeftMiddleProximal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.075, 0.0]))));
        this.insert(BoneName::LeftMiddleIntermediate, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.075, 0.0]))));
        this.insert(BoneName::LeftMiddleDistal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.025, 0.0]))));
        this.insert(BoneName::LeftRingProximal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.025, 0.075, 0.0]))));
        this.insert(BoneName::LeftRingIntermediate, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.05, 0.0]))));
        this.insert(BoneName::LeftRingDistal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.025, 0.0]))));
        this.insert(BoneName::LeftLittleProximal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.05, 0.05, 0.0]))));
        this.insert(BoneName::LeftLittleIntermediate, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.05, 0.0]))));
        this.insert(BoneName::LeftLittleDistal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.025, 0.0]))));
        this.insert(BoneName::RightShoulder, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[0.0, 0.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, -0.05, 0.1, 0.0]))));
        this.insert(BoneName::RightUpperArm, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[-1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.05, 0.0]))));
        this.insert(BoneName::RightLowerArm, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[0.0, 0.0, -1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.25, 0.0]))));
        this.insert(BoneName::RightHand, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[0.0, 0.0, 1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.25, 0.0]))));
        this.insert(BoneName::RightThumbProximal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[0.0, 0.0, 1.0, 0.577, 0.816, 0.0, -0.816, 0.577, 0.0, 0.025, 0.025, 0.0]))));
        this.insert(BoneName::RightThumbIntermediate, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.043, 0.0]))));
        this.insert(BoneName::RightThumbDistal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.043, 0.0]))));
        this.insert(BoneName::RightIndexProximal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.025, 0.075, 0.0]))));
        this.insert(BoneName::RightIndexIntermediate, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.05, 0.0]))));
        this.insert(BoneName::RightIndexDistal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.025, 0.0]))));
        this.insert(BoneName::RightMiddleProximal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.075, 0.0]))));
        this.insert(BoneName::RightMiddleIntermediate, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.075, 0.0]))));
        this.insert(BoneName::RightMiddleDistal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.025, 0.0]))));
        this.insert(BoneName::RightRingProximal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, -0.025, 0.075, 0.0]))));
        this.insert(BoneName::RightRingIntermediate, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.05, 0.0]))));
        this.insert(BoneName::RightRingDistal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.025, 0.0]))));
        this.insert(BoneName::RightLittleProximal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, -0.05, 0.05, 0.0]))));
        this.insert(BoneName::RightLittleIntermediate, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.05, 0.0]))));
        this.insert(BoneName::RightLittleDistal, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.025, 0.0]))));
        this.insert(BoneName::LeftUpperLeg, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[-1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.1, 0.0, 0.0]))));
        this.insert(BoneName::LeftLowerLeg, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[-1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.375, 0.0]))));
        this.insert(BoneName::LeftFoot, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[-1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, -1.0, 0.0, 0.0, 0.375, 0.0]))));
        this.insert(BoneName::LeftToes, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[-1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.15, 0.0]))));
        this.insert(BoneName::RightUpperLeg, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[-1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0, -0.1, 0.0, 0.0]))));
        this.insert(BoneName::RightLowerLeg, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[-1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.375, 0.0]))));
        this.insert(BoneName::RightFoot, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[-1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, -1.0, 0.0, 0.0, 0.375, 0.0]))));
        this.insert(BoneName::RightToes, Transform::from_matrix(Mat4::from(Affine3A::from_cols_array(&[-1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.15, 0.0]))));
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