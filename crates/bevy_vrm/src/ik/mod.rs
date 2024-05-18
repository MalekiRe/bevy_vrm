use std::f32::consts::{PI, TAU};
use bevy::app::App;
use bevy::ecs::query::QueryEntityError;
use bevy::ecs::system::RunSystemOnce;
use bevy::gltf::GltfNode;
use bevy::math::EulerRot::XYZ;
use bevy::prelude::*;
use bevy_inspector_egui::InspectorOptions;
use serde_vrm::vrm0::BoneName;
use crate::HumanoidBones;
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use random_number::random;
use crate::retargeting::{RunBoneRestEvent, VrmRetargetingInitialized};

pub struct RenIkPlugin;

impl Plugin for RenIkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, perform_hand_left_ik);
        app.add_systems(PreUpdate, add_bone_rest);
        app.add_systems(Update, add_target);
        //app.add_systems(Update, update_test);
    }
}

#[derive(Component)]
struct DrawableTarget;


fn add_target(mut commands: Commands, skeletons: Query<(Entity, &HumanoidBones), (Without<Target>, With<VrmRetargetingInitialized>)>, mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<StandardMaterial>>,) {
    for (entity, _) in skeletons.iter() {
        let id = commands.spawn((
            PbrBundle {
                mesh: meshes.add(Sphere::new(0.05)),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgba(0.8, 0.8, 0.8, 1.0),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                }),
                transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(Vec3::new(0.136, 1.015, 0.056)),
                ..Default::default()
            },
            bevy_mod_picking::PickableBundle::default(),
            bevy_transform_gizmo::GizmoTransformable,
            DrawableTarget,
        )).id();
        commands.entity(entity).insert(Target {
            left_hand: id,
        });
        commands.entity(entity).insert(RenikLimb::default());
    }
}


#[derive(Component, Clone, Copy)]
pub struct Target {
    pub left_hand: Entity,
}

#[derive(Component)]
pub struct BoneRest(pub Transform);

fn add_bone_rest(mut commands: Commands, query: Query<(Entity, &Transform), Without<BoneRest>>, mut event_reader: EventReader<RunBoneRestEvent>) {
    for e in event_reader.read() {
        println!("setting event rest");
        for (entity, transform) in query.iter() {
            commands.entity(entity).insert(BoneRest(transform.clone()));
        }
    }
}
const LEFT_SHOULDER_OFFSET: Quat = Quat::IDENTITY;

const ARM_SHOULDER_INFLUENCE: f32 = 0.25;

fn perform_hand_left_ik(mut event_reader: EventReader<RunBoneRestEvent>,
                        mut run: Local<bool>, skeletons: Query<(&HumanoidBones, &Target, &GlobalTransform, &RenikLimb), With<VrmRetargetingInitialized>>, parents: Query<&Parent>, bone_rests: Query<&BoneRest>, global_transforms: Query<&GlobalTransform>,
mut local_transforms: Query<&mut Transform>) {

    let mut temp = false;
    for e in event_reader.read() {
        *run = true;
        temp = true;
    }
    if !*run || temp {
        return;
    }

    for (skeleton, target, root_global_transform, renik_limb) in skeletons.iter() {
        let Some(left_upper_arm) = skeleton.0.get(&BoneName::LeftUpperArm) else { continue; };
        let left_upper_arm_parent = parents.get(*left_upper_arm).expect("missing parent of left upper arm").get();

        let target = global_transforms.get(target.left_hand).unwrap().compute_transform();

        //println!("{}", root_global_transform.translation());
        //println!("{}", root_global_transform.to_scale_rotation_translation().1);
        //println!("{}", target.translation);
        //println!("{}", target.rotation);

        let target = Transform::from_matrix(root_global_transform.compute_matrix().inverse() * target.compute_matrix());

        //let target = Transform::from_matrix(root_global_transform.compute_matrix().inverse() * global_transforms.get(target.left_hand).unwrap().compute_matrix());

        //println!("target: {}", target.translation);

        let root = Transform::from_translation(Vec3::new(0.0, 1.129259, 0.006192))
            .with_rotation(Quat::from_euler(XYZ, -0.054638, 0.0, 0.0))
            .with_scale(Vec3::splat(1.0));

        let mut root = root * match bone_rests.get(left_upper_arm_parent) {
            Ok(a) => a.0,
            Err(_) => { continue; },
        };

        //println!("origin: {}", root.translation);
        //println!("rotation: {:?}", root.rotation.to_euler(XYZ));

        let target_vector = root.compute_matrix().inverse().transform_point3(target.translation);
        //println!("target vector: {}", target_vector);



        let offset_quat = renik_limb.left_shoulder_offset;
        let pole_offset = renik_limb.shoulder_pole_offset.clone();
        //println!("a: {:?}, b: {:?}", offset_quat.to_euler(XYZ), pole_offset.to_euler(XYZ));
        let pole_offset_scaled = pole_offset.slerp(Quat::IDENTITY, 1.0 - ARM_SHOULDER_INFLUENCE);

        //println!("{:?}", pole_offset_scaled.to_euler(XYZ));

        //println!("pole offset scaled: {:?}, offset_quat: {:?}, target_vector: {:?}, arm_shoulder_influence: {:?}, pole_offset: {:?}",
        //pole_offset_scaled.to_euler(XYZ), offset_quat.to_euler(XYZ), target_vector, ARM_SHOULDER_INFLUENCE, pole_offset.to_euler(XYZ));

        let quat_align_to_target = pole_offset_scaled * align_vectors(
            Vec3::new(0.0, 1.0, 0.0),
            pole_offset.inverse() * ( offset_quat.inverse() * target_vector ),
                1.0,
        ).slerp(Quat::IDENTITY, 1.0 - ARM_SHOULDER_INFLUENCE);

        //println!("quat align to target: {:?}", quat_align_to_target.to_euler(XYZ));

        let custom_pose = Transform::from_rotation(offset_quat * quat_align_to_target);
        let mut current_transform = local_transforms.get_mut(left_upper_arm_parent).unwrap();
        let rest_transform = bone_rests.get(left_upper_arm_parent).unwrap().0.clone();

        current_transform.rotation = rest_transform.rotation * offset_quat * quat_align_to_target;

        let root = root * custom_pose;


        //println!("origin: {}", root.translation);
        //println!("rotation: {:?}", root.rotation.to_euler(XYZ));



        //println!("{}", target.translation);

        do_ik_bullshit(renik_limb.clone(), root, target, &bone_rests, &skeleton, &mut local_transforms);

    }
}

pub trait PoleOffset {
    fn pole_offset(&self) -> Quat;
}

pub fn left_arm_pole_offset() -> Quat {
    Quat::from_euler(EulerRot::XYZ, 15.0f32.to_radians(), 0.0, 60.0f32.to_radians())
}

#[derive(Debug, Clone, Component)]
#[derive(Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct RenikLimb {
    upper_twist_offset: f32,
    lower_twist_offset: f32,
    roll_offset: f32,
    upper_limb_twist: f32,
    lower_limb_twist: f32,
    twist_inflection_point_offset: f32,
    twist_overflow: f32,
    target_rotation_influence: f32,
    pole_offset: Quat,
    target_position_influence: Vec3,
    pub shoulder_pole_offset: Quat,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    left_shoulder_offset: Quat,
}

impl Default for RenikLimb {
    fn default() -> Self {
       RenikLimb {
            upper_twist_offset: -0.27777 * PI,
            lower_twist_offset: -0.27777 * PI,
            roll_offset: (-70.0_f32).to_radians(),
            upper_limb_twist: 0.5,
            lower_limb_twist: 0.66666,
            twist_inflection_point_offset: 20.0_f32.to_radians(),
            twist_overflow: 45.0_f32.to_radians(),
            target_rotation_influence: 0.33,
            pole_offset: Quat::from_euler(
                EulerRot::XYZ,
                15.0_f32.to_radians(),
                0.0,
                60.0_f32.to_radians()
            ),
            shoulder_pole_offset:/* Quat::from_euler(XYZ, 0.0, 0.0, 78f32.to_radians())*/ Quat::IDENTITY,
            a: Vec3::new(0.0, 1.0, 0.0),
            b: Vec3::new(1.0, 0.0, 0.0),
            target_position_influence: Vec3::new(2.0, -1.5, -1.0),
            c: Vec3::new(0.0, 1.0, 0.0),
            left_shoulder_offset: Quat::IDENTITY,
        }

    }
}

static mut LEFT_ARM_OVERFLOW_STATE: f32 = 0.0;

fn do_ik_bullshit(limb: RenikLimb, root: Transform, local_target: Transform, bone_rests: &Query<&BoneRest>, skeleton: &HumanoidBones, local_transforms: &mut Query<&mut Transform>) {

    let true_root = root * Transform::from_translation(bone_rests.get(*skeleton.0.get(&BoneName::LeftUpperArm).unwrap()).unwrap().0.translation);

    //println!("{}, {}", local_target.rotation, local_target.translation);

    let local_target = Transform::from_matrix(true_root.compute_matrix().inverse() * local_target.compute_matrix());


    let mut full_upper = Transform::from_translation(bone_rests.get(*skeleton.0.get(&BoneName::LeftUpperArm).unwrap()).unwrap().0.clone().translation);
    let mut full_lower = Transform::from_translation(bone_rests.get(*skeleton.0.get(&BoneName::LeftLowerArm).unwrap()).unwrap().0.clone().translation);
    let mut leaf = Transform::from_translation(bone_rests.get(*skeleton.0.get(&BoneName::LeftHand).unwrap()).unwrap().0.clone().translation);

    let upper_vector = full_lower.translation.clone();
    let lower_vector = leaf.translation.clone();


    let mut target_vector = local_target.translation;

    //println!("{}", target_vector);

    let mut normalized_target_vector = target_vector.normalize();
    //TODO remove this
    //normalized_target_vector = Vec3::new(0.213173, 0.518952, -0.827796);

    let limb_length = upper_vector.length() + lower_vector.length();
    if target_vector.length() > upper_vector.length() + lower_vector.length() {
        target_vector = normalized_target_vector * limb_length;
    }

    //println!("{}", target_vector);

    let mut angles = trig_angles(upper_vector, lower_vector, target_vector);

    //println!("{}", angles);

    let starting_pole = limb.pole_offset * /*Vec3::new(0.0, 1.0, 0.0)*/ limb.a;
    let mut joint_axis = align_vectors(starting_pole, target_vector, 1.0) * (limb.pole_offset * Vec3::new(1.0, 0.0, 0.0));

    //println!("{}", starting_pole);

    //joint_axis = Vec3::new(0.741801, 0.465429, 0.482809);

    //println!("{}", joint_axis);

    let leaf_rest_vector = full_upper.rotation * ( full_lower * leaf.translation);
    let positional_offset = limb.target_position_influence.dot(target_vector - leaf_rest_vector);
    joint_axis = Quat::from_axis_angle(normalized_target_vector, positional_offset + limb.roll_offset).mul_vec3(joint_axis);


    //println!("{}", joint_axis);
    let local_leaf_vector = local_target.rotation * Vec3::new(0.0, 1.0, 0.0);
    let local_lower_vector = Quat::from_axis_angle(joint_axis, angles.x - angles.y).mul_vec3(normalized_target_vector).normalize();


    let leaf_rejection = vector_rejection(local_leaf_vector, normalized_target_vector);
    let lower_rejection = vector_rejection(local_lower_vector, normalized_target_vector);



    let mut joint_roll_amount = leaf_rejection.angle_between(lower_rejection) * limb.target_rotation_influence;

    if lower_rejection.length() <= 0.001 {
        joint_roll_amount = 0.0;
    }

    joint_roll_amount *= local_leaf_vector.cross(local_lower_vector).dot(normalized_target_vector).abs();


    if leaf_rejection.cross(lower_rejection).dot(normalized_target_vector) > 0.0 {
        joint_roll_amount *= -1.0;
    }

    //println!("{}", joint_roll_amount);

    joint_axis = Quat::from_axis_angle(normalized_target_vector, joint_roll_amount).mul_vec3(joint_axis);

    //println!("{}", joint_axis);

    let total_roll = joint_roll_amount + positional_offset + limb.roll_offset;

    let leaf_x = align_vectors(
        Quat::from_axis_angle(normalized_target_vector, joint_roll_amount).mul_vec3(local_leaf_vector),
        Quat::from_axis_angle(normalized_target_vector, joint_roll_amount).mul_vec3(local_lower_vector),
    1.0)
        * (local_target.rotation * Vec3::new(1.0, 0.0, 0.0));
    let rolled_joint_axis = Quat::from_axis_angle(local_lower_vector, -1.0 * total_roll).mul_vec3(joint_axis);
    let lower_z = rolled_joint_axis.cross(local_lower_vector);
    let mut twist_angle = leaf_x.angle_between(rolled_joint_axis);
    if leaf_x.dot(lower_z) > 0.0 {
        twist_angle *= -1.0;
    }


    let mut inflection_point = if twist_angle > 0.0 { PI } else { -PI } - limb.twist_inflection_point_offset;
    let overflow_area = unsafe { LEFT_ARM_OVERFLOW_STATE } * limb.twist_overflow;
    let inflection_distance = twist_angle - inflection_point;

    if inflection_distance.abs() < limb.twist_overflow {
        if unsafe { LEFT_ARM_OVERFLOW_STATE } == 0.0 {
            if inflection_distance < 0.0 {
                unsafe { LEFT_ARM_OVERFLOW_STATE = 1.0 }
            } else {
                unsafe { LEFT_ARM_OVERFLOW_STATE = -1.0 }
            }
        }
    } else {
       unsafe { LEFT_ARM_OVERFLOW_STATE = 0.0}
    }

    inflection_point += overflow_area;
    if twist_angle > 0.0 && twist_angle > inflection_point {
        twist_angle -= TAU;
    }
    else if twist_angle < 0.0 && twist_angle < inflection_point {
        twist_angle += TAU;
    }

    let mut lower_twist = twist_angle * limb.lower_limb_twist;
    let upper_twist = lower_twist * limb.upper_limb_twist + limb.upper_twist_offset - total_roll;

    lower_twist += limb.lower_twist_offset - 2.0 * limb.roll_offset - positional_offset - joint_roll_amount;

    joint_axis = Quat::from_axis_angle(normalized_target_vector, twist_angle * limb.target_rotation_influence).mul_vec3(joint_axis);
    // Rebuild the rotations

    //joint_axis = Vec3::new(0.017751, 0.845073, 0.534354);
    //normalized_target_vector = Vec3::new(0.213173, 0.518952, -0.827796);
    //angles.x = 0.94497638940811;
    //angles.y = 1.8366014957428;
    //println!("{}", angles.y);
    //println!("{}", angles.x);

    //println!("{}", joint_axis);
    //println!("{}", normalized_target_vector);


    let upper_joint_vector = Quat::from_axis_angle(joint_axis, angles.x).mul_vec3(normalized_target_vector);
    let a = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), -limb.roll_offset);


    let rolled_lower_joint_axis = a * Vec3::new(1.0, 0.0, 0.0);
    //println!("{}", rolled_lower_joint_axis);
    let lower_joint_vector = Quat::from_axis_angle(rolled_lower_joint_axis, angles.y).mul_vec3(Vec3::new(0.0, 1.0, 0.0));
    let twisted_joint_axis = Quat::from_axis_angle(upper_joint_vector, upper_twist).mul_vec3(joint_axis);
    let upper_basis = Quat::from_mat3(&Mat3::from_cols(twisted_joint_axis, upper_joint_vector, twisted_joint_axis.cross(upper_joint_vector)));
    //println!("{:?}", lower_joint_vector);
    let mut lower_basis = Quat::from_mat3(&Mat3::from_cols(rolled_lower_joint_axis, lower_joint_vector, rolled_lower_joint_axis.cross(lower_joint_vector)));

    //println!("{}", lower_joint_vector);

    //println!("{:?}", lower_basis.to_euler(XYZ));

    lower_basis = transpose_quat(lower_basis);

    //println!("{:?}", lower_basis.to_euler(XYZ));

    lower_basis = lower_basis * Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), lower_twist);

    //println!("{:?}", lower_basis.to_euler(XYZ));

    //println!("{:?}", upper_twist);

    lower_basis = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), -upper_twist) * lower_basis;

    //rotate_quat(lower_basis, Vec3::new(0.0, 1.0, 0.0), -upper_twist);
    //println!("{:?}", lower_basis.to_euler(XYZ));

    //println!("{:?}", full_lower.rotation);

    let upper_transform = (full_upper.rotation.inverse() * upper_basis).normalize();
    let lower_transform = (full_lower.rotation.inverse() * lower_basis).normalize();
    let leaf_transform = (leaf.rotation.inverse() * (upper_basis * lower_basis).inverse() * local_target.rotation * leaf.rotation);

    //println!("upper transform: {:?}", upper_transform.to_euler(XYZ));
    //println!("lower transform: {:?}", lower_transform.to_euler(XYZ));
    //println!("leaf_transform: {:?}", leaf_transform.to_euler(XYZ));

    local_transforms.get_mut(*skeleton.0.get(&BoneName::LeftUpperArm).unwrap()).unwrap().rotation = upper_transform;
    local_transforms.get_mut(*skeleton.0.get(&BoneName::LeftLowerArm).unwrap()).unwrap().rotation = lower_transform;
    local_transforms.get_mut(*skeleton.0.get(&BoneName::LeftHand).unwrap()).unwrap().rotation = leaf_transform;



}

/*fn rotate_quat(lower_basis: Quat, axis: Vec3, angle_in_degrees: f32) -> Quat {
    // Convert degrees to radians
    let angle_in_radians = angle_in_degrees.to_radians();

    // Create a rotation quaternion around the specified axis and angle
    let rotation_quat = Quat::from_axis_angle(axis.normalize(), angle_in_radians);

    // Apply the rotation by multiplying the quaternions
    rotation_quat * lower_basis
}*/


fn transpose_quat(quat: Quat) -> Quat {
    // Convert the quaternion to a matrix
    let mat = Mat3::from_quat(quat);

    // Transpose the matrix
    let transposed_mat = mat.transpose();

    // Convert the transposed matrix back to a quaternion
    Quat::from_mat3(&transposed_mat)
}

fn vector_rejection(v: Vec3, normal: Vec3) -> Vec3 {
    if v.length_squared() == 0.0 || normal.length_squared() == 0.0 {
        Vec3::default()
    } else {
        let normal_length = normal.length();
        let proj = (normal.dot(v) / normal_length) * (normal / normal_length);
        v - proj
    }
}

fn trig_angles(side1: Vec3, side2: Vec3, side3: Vec3) -> Vec2 {
    let length1_squared = side1.length_squared();
    let length2_squared = side2.length_squared();
    let length3_squared = side3.length_squared();

    let length1 = length1_squared.sqrt() * 2.0;
    let length2 = length2_squared.sqrt();
    let length3 = length3_squared.sqrt();

    let angle1 = safe_acos((length1_squared + length3_squared - length2_squared) / ( length1 * length3));
    let angle2 = PI - safe_acos((length1_squared + length2_squared - length3_squared) / ( length1 * length2));
    Vec2::new(angle1, angle2)
}

fn safe_acos(f: f32) -> f32 {
    f.clamp(-1.0, 1.0).acos()
}

/// Default of 1
fn align_vectors(mut a: Vec3, mut b: Vec3, influence: f32) -> Quat {
    if a.length() == 0.0 || b.length() == 0.0 {
        return Quat::IDENTITY;
    }

    a = a.normalize();
    b = b.normalize();

    if a.length_squared() != 0.0 && b.length_squared() != 0.0 {
        let mut perpendicular = a.cross(b);
        let angle_diff = a.angle_between(b) * influence;


        if perpendicular.length_squared() == 0.0
        {
            perpendicular = get_perpendicular_vector(a);
        }
        let a = perpendicular.normalize().normalize();
        let ret = Quat::from_axis_angle(a, angle_diff);

        ret.normalize()
    } else {
        Quat::IDENTITY
    }
}

fn get_perpendicular_vector(v: Vec3) -> Vec3 {
    if v.x != 0.0 && v.y != 0.0 {
        Vec3::new(0.0, 0.0, 1.0).cross(v).normalize()
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    }
}