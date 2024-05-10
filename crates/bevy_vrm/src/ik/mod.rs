use bevy::app::App;
use std::f32::consts::{PI, TAU};

use bevy::ecs::system::RunSystemOnce;

use crate::retargeting::VrmRetargetingInitialized;
use crate::HumanoidBones;
use bevy::math::EulerRot::XYZ;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use bevy_inspector_egui::InspectorOptions;
use random_number::random;
use serde_vrm::vrm0::BoneName;

pub struct RenIkPlugin;

impl Plugin for RenIkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, perform_hand_left_ik);
        app.add_systems(Update, add_bone_rest);
        app.add_systems(Update, add_target);
        //app.add_systems(Update, update_test);
    }
}

#[derive(Component)]
struct DrawableTarget;

static mut PREVIOUS_BEST: f32 = 100000.0;

fn update_test(world: &mut World) {
    for _ in 0..300 {
        let old_limb: Option<RenikLimb> =
            world.run_system_once_with((), |mut limb: Query<&mut RenikLimb>| {
                let Ok(limb) = limb.get_single_mut() else {
                    return None;
                };
                Some(limb.clone())
            });

        let Some(old_limb) = old_limb else { return };

        world.run_system_once_with(
            old_limb.clone(),
            |old_limb: In<RenikLimb>, mut limb: Query<&mut RenikLimb>| {
                let mut limb = limb.get_single_mut().unwrap();
                //limb.a = Vec3::new(random!(-1.0..1.0), random!(-1.0..1.0), random!(-1.0..1.0)).normalize();
                //limb.b = Vec3::new(random!(-1.0..1.0), random!(-1.0..1.0), random!(-1.0..1.0)).normalize();
                //limb.c = Vec3::new(random!(-1.0..1.0), random!(-1.0..1.0), random!(-1.0..1.0)).normalize();
                match (random!(0..14), random!(0..3)) {
                    (0, 0) => {
                        let (_x, y, z) = old_limb.pole_offset.to_euler(XYZ);
                        limb.pole_offset = Quat::from_euler(XYZ, random!(0.0..360.0), y, z);
                    }
                    (0, 1) => {
                        let (x, _y, z) = old_limb.pole_offset.to_euler(XYZ);
                        limb.pole_offset = Quat::from_euler(XYZ, x, random!(0.0..360.0), z);
                    }
                    (0, 2) => {
                        let (x, y, _z) = old_limb.pole_offset.to_euler(XYZ);
                        limb.pole_offset = Quat::from_euler(XYZ, x, y, random!(0.0..360.0));
                    }
                    (1, _) => {
                        limb.roll_offset = random!(-7.0..7.0);
                    }
                    (2, _) => {
                        limb.lower_twist_offset = random!(-7.0..7.0);
                    }
                    (3, _) => {
                        limb.upper_twist_offset = random!(-7.0..7.0);
                    }
                    (4, _) => {
                        limb.twist_inflection_point_offset = random!(-7.0..7.0);
                    }
                    (5, _) => {
                        limb.twist_overflow = random!(-7.0..7.0);
                    }
                    (6, _) => {
                        limb.twist_overflow = random!(-7.0..7.0);
                    }
                    (7, _) => {
                        limb.upper_limb_twist = random!(-7.0..7.0);
                    }
                    (8, _) => {
                        limb.lower_limb_twist = random!(-7.0..7.0);
                    }
                    /*(9, 0) => {
                        limb.target_position_influence = Vec3::new(random!(-5.0..5.0), limb.target_position_influence.y, limb.target_position_influence.z);
                    }
                    (9, 1) => {
                        limb.target_position_influence = Vec3::new(limb.target_position_influence.x, random!(-5.0..5.0), limb.target_position_influence.z);
                    }
                    (9, 2) => {
                        limb.target_position_influence = Vec3::new(limb.target_position_influence.x, limb.target_position_influence.y, random!(-5.0..5.0));
                    }*/
                    (10, 0) => {
                        let (_x, y, z) = old_limb.shoulder_pole_offset.to_euler(XYZ);
                        limb.shoulder_pole_offset =
                            Quat::from_euler(XYZ, random!(0.0..360.0), y, z);
                    }
                    (10, 1) => {
                        let (x, _y, z) = old_limb.shoulder_pole_offset.to_euler(XYZ);
                        limb.shoulder_pole_offset =
                            Quat::from_euler(XYZ, x, random!(0.0..360.0), z);
                    }
                    (10, 2) => {
                        let (x, y, _z) = old_limb.shoulder_pole_offset.to_euler(XYZ);
                        limb.shoulder_pole_offset =
                            Quat::from_euler(XYZ, x, y, random!(0.0..360.0));
                    }
                    (11, _) => {
                        limb.a = match random!(0..6) {
                            0 => Vec3::NEG_X,
                            1 => Vec3::NEG_Y,
                            2 => Vec3::NEG_Z,
                            3 => Vec3::X,
                            4 => Vec3::Y,
                            5 => Vec3::Z,
                            _ => Vec3::X,
                        };
                    }
                    (12, _) => {
                        limb.b = match random!(0..6) {
                            0 => Vec3::NEG_X,
                            1 => Vec3::NEG_Y,
                            2 => Vec3::NEG_Z,
                            3 => Vec3::X,
                            4 => Vec3::Y,
                            5 => Vec3::Z,
                            _ => Vec3::X,
                        };
                    }
                    (13, _) => {
                        limb.c = match random!(0..6) {
                            0 => Vec3::NEG_X,
                            1 => Vec3::NEG_Y,
                            2 => Vec3::NEG_Z,
                            3 => Vec3::X,
                            4 => Vec3::Y,
                            5 => Vec3::Z,
                            _ => Vec3::X,
                        };
                    }
                    /*(11, 0) => {
                        limb.a = Vec3::new(random!(-1.0..1.0), limb.a.y, limb.a.z).normalize();
                    }
                    (11, 1) => {
                        limb.a = Vec3::new(limb.a.x, random!(-1.0..1.0), limb.a.z).normalize();
                    }
                    (11, 2) => {
                        limb.a = Vec3::new(limb.a.x, limb.a.y, random!(-1.0..1.0)).normalize();
                    }
                    (12, 0) => {
                        limb.b = Vec3::new(random!(-1.0..1.0), limb.b.y, limb.b.z).normalize();
                    }
                    (12, 1) => {
                        limb.b = Vec3::new(limb.b.x, random!(-1.0..1.0), limb.b.z).normalize();
                    }
                    (12, 2) => {
                        limb.b = Vec3::new(limb.b.x, limb.b.y, random!(-1.0..1.0)).normalize();
                    }
                    (13, 0) => {
                        limb.c = Vec3::new(random!(-1.0..1.0), limb.c.y, limb.c.z).normalize();
                    }
                    (13, 1) => {
                        limb.c = Vec3::new(limb.c.x, random!(-1.0..1.0), limb.c.z).normalize();
                    }
                    (13, 2) => {
                        limb.c = Vec3::new(limb.c.x, limb.c.y, random!(-1.0..1.0)).normalize();
                    }*/
                    (14, 0) => {
                        let (_x, y, z) = old_limb.left_shoulder_offset.to_euler(XYZ);
                        limb.left_shoulder_offset =
                            Quat::from_euler(XYZ, random!(0.0..360.0), y, z);
                    }
                    (14, 1) => {
                        let (x, _y, z) = old_limb.left_shoulder_offset.to_euler(XYZ);
                        limb.left_shoulder_offset =
                            Quat::from_euler(XYZ, x, random!(0.0..360.0), z);
                    }
                    (14, 2) => {
                        let (x, y, _z) = old_limb.left_shoulder_offset.to_euler(XYZ);
                        limb.left_shoulder_offset =
                            Quat::from_euler(XYZ, x, y, random!(0.0..360.0));
                    }
                    _ => (),
                }
            },
        );

        let mut running_total = 0.0;

        let ball_iterations = 25.0;

        for _ in 0..ball_iterations as usize {
            world.run_system_once(|mut ball: Query<&mut Transform, With<DrawableTarget>>| {
                ball.get_single_mut().unwrap().translation =
                    Vec3::new(random!(-2.0..2.0), random!(-2.0..2.0), random!(-2.0..2.0));
            });
            world.run_system_once(bevy::transform::systems::propagate_transforms);

            world.run_system_once(perform_hand_left_ik);

            world.run_system_once(bevy::transform::systems::propagate_transforms);

            let distance: f32 = world.run_system_once_with(
                (),
                |query: Query<&RenikLimb>,
                 ball: Query<Entity, With<DrawableTarget>>,
                 query2: Query<&HumanoidBones>,
                 transforms: Query<&GlobalTransform>|
                 -> f32 {
                    let _q = query.get_single().unwrap();
                    let ball = ball.single();
                    let q2 = query2.single();
                    let Some(temp) = q2.0.get(&BoneName::LeftHand) else {
                        return 1000.0;
                    };
                    let Ok(hand_pos) = transforms.get(*temp) else {
                        return 1000.0;
                    };
                    let Ok(ball_pos) = transforms.get(ball) else {
                        return 1000.0;
                    };
                    hand_pos.translation().distance(ball_pos.translation())
                },
            );
            running_total += distance;
        }
        running_total /= ball_iterations;

        if running_total < unsafe { PREVIOUS_BEST } {
            unsafe { PREVIOUS_BEST = running_total };
            world.run_system_once_with((), |limb: Query<&mut RenikLimb>| {
                let limb = limb.get_single().unwrap();
                println!("new best limb: {:#?}", limb);
            });
        } else {
            world.run_system_once_with(
                old_limb.clone(),
                |old_limb: In<RenikLimb>, mut limb: Query<&mut RenikLimb>| {
                    *limb.get_single_mut().unwrap() = old_limb.clone();
                },
            );
        }
    }
}

fn add_target(
    mut commands: Commands,
    skeletons: Query<(Entity, &HumanoidBones), Without<Target>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, _) in skeletons.iter() {
        let id = commands
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Sphere::new(0.1)),
                    material: materials.add(Color::rgb(0.8, 0.8, 0.8)),
                    transform: Transform::from_scale(Vec3::splat(1.0))
                        .with_translation(Vec3::new(-1.0, 0.0, -1.0)),
                    ..Default::default()
                },
                bevy_mod_picking::PickableBundle::default(),
                bevy_transform_gizmo::GizmoTransformable,
                DrawableTarget,
            ))
            .id();
        commands.entity(entity).insert(Target { left_hand: id });
        commands.entity(entity).insert(RenikLimb::default());
    }
}

#[derive(Component, Clone, Copy)]
pub struct Target {
    pub left_hand: Entity,
}

#[derive(Component)]
pub struct BoneRest(pub Transform);

fn add_bone_rest(mut commands: Commands, query: Query<(Entity, &Transform), Without<BoneRest>>) {
    for (entity, transform) in query.iter() {
        commands.entity(entity).insert(BoneRest(*transform));
    }
}

fn adjust_pole_offset(
    mut query: Query<&mut RenikLimb>,
    ball: Query<Entity, With<DrawableTarget>>,
    mut local: Local<(RenikLimb, f32)>,
    query2: Query<&HumanoidBones>,
    transforms: Query<&GlobalTransform>,
    mut locals: Query<&mut Transform>,
) {
    let Ok(mut q) = query.get_single_mut() else {
        return;
    };
    let ball = ball.single();
    let q2 = query2.single();
    let Some(temp) = q2.0.get(&BoneName::LeftHand) else {
        return;
    };
    let Ok(hand_pos) = transforms.get(*temp) else {
        return;
    };
    let Ok(ball_pos) = transforms.get(ball) else {
        return;
    };

    if local.1 == 0.0 {
        local.1 = 1000.0;
    }

    if hand_pos.translation().distance(ball_pos.translation()) < local.1 {
        local.0 = q.clone();
        local.1 = hand_pos.translation().distance(ball_pos.translation());
        println!("new best: {:#?}", q);
    }

    let (_x, _y, _z) = q.pole_offset.to_euler(XYZ);

    q.pole_offset = Quat::from_euler(
        XYZ,
        random!(0.0..360.0),
        random!(0.0..360.0),
        random!(0.0..360.0),
    );
    q.shoulder_pole_offset = Quat::from_euler(
        XYZ,
        random!(0.0..360.0),
        random!(0.0..360.0),
        random!(0.0..360.0),
    );
    /*
    match random!(0..1) {
        0 => {
            q.pole_offset = Quat::from_euler(XYZ, random!(0.0..360.0), random!(0.0..360.0),random!(0.0..360.0));
        }
        /*1 => {
            q.roll_offset = random!(0.0..7.0);
        }
        2 => {
            q.lower_twist_offset = random!(0.0..7.0);
        }
        3 => {
            q.upper_twist_offset = random!(0.0..7.0);
        }
        4 => {
            q.twist_inflection_point_offset = random!(0.0..7.0);
        }
        5 => {
            q.twist_overflow = random!(0.0..7.0);
        }
        6 => {
            q.twist_overflow = random!(0.0..7.0);
        }
        7 => {
            q.upper_limb_twist = random!(0.0..7.0);
        }
        8 => {
            q.lower_limb_twist = random!(0.0..7.0);
        }
        9 => {
            q.target_position_influence = Vec3::new(random!(-5.0..5.0), random!(-5.0..5.0), random!(-5.0..5.0));
        }*/
        _ => return,
    }*/

    let _selected = random!(0..3);
    /*let mut new_ball_pos = match selected {
        0 => Vec3::X,
        1 => Vec3::Y,
        2 => Vec3::Z,
        _ => return,
    };

    if random!(0..1) == 1 {
        new_ball_pos *= -1.0;
    }*/

    let new_ball_pos =
        Vec3::new(random!(-2.0..2.0), random!(-2.0..2.0), random!(-2.0..2.0)).normalize();

    locals.get_mut(ball).unwrap().translation = new_ball_pos;
}

const LEFT_SHOULDER_OFFSET: Quat = Quat::IDENTITY;

fn left_shoulder_pole_offset() -> Quat {
    Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 78f32.to_radians())
}

const ARM_SHOULDER_INFLUENCE: f32 = 0.25;

fn perform_hand_left_ik(
    _local: Local<i32>,
    skeletons: Query<
        (&HumanoidBones, &Target, &GlobalTransform, &RenikLimb),
        With<VrmRetargetingInitialized>,
    >,
    parents: Query<&Parent>,
    bone_rests: Query<&BoneRest>,
    global_transforms: Query<&GlobalTransform>,
    mut local_transforms: Query<&mut Transform>,
) {
    for (skeleton, target, root_global_transform, renik_limb) in skeletons.iter() {
        let Some(left_upper_arm) = skeleton.0.get(&BoneName::LeftUpperArm) else {
            continue;
        };
        let left_upper_arm_parent = parents
            .get(*left_upper_arm)
            .expect("missing parent of left upper arm")
            .get();

        let target = Transform::from_matrix(
            root_global_transform.compute_matrix().inverse()
                * global_transforms
                    .get(target.left_hand)
                    .unwrap()
                    .compute_matrix(),
        );

        let _root = global_transforms
            .get(left_upper_arm_parent)
            .unwrap()
            .compute_transform();

        let root = global_transforms
            .get(left_upper_arm_parent)
            .unwrap()
            .compute_transform()
            * match bone_rests.get(left_upper_arm_parent) {
                Ok(a) => a.0,
                Err(_) => {
                    continue;
                }
            };
        //println!("origin: {}", root.translation);
        //println!("rotation: {:?}", root.rotation.to_euler(XYZ));

        let target_vector = root
            .compute_matrix()
            .inverse()
            .transform_point3(target.translation);
        //println!("target vector: {}", target_vector);
        let offset_quat = renik_limb.left_shoulder_offset;
        let pole_offset = renik_limb.shoulder_pole_offset;
        //println!("a: {:?}, b: {:?}", offset_quat.to_euler(XYZ), pole_offset.to_euler(XYZ));
        let pole_offset_scaled = pole_offset.slerp(Quat::IDENTITY, 1.0 - ARM_SHOULDER_INFLUENCE);

        //println!("{:?}", pole_offset_scaled.to_euler(XYZ));

        //println!("pole offset scaled: {:?}, offset_quat: {:?}, target_vector: {:?}, arm_shoulder_influence: {:?}, pole_offset: {:?}",
        //pole_offset_scaled.to_euler(XYZ), offset_quat.to_euler(XYZ), target_vector, ARM_SHOULDER_INFLUENCE, pole_offset.to_euler(XYZ));

        let quat_align_to_target = pole_offset_scaled
            * align_vectors(
                Vec3::new(0.0, 1.0, 0.0),
                pole_offset.inverse() * (offset_quat.inverse() * target_vector),
                1.0,
            )
            .slerp(Quat::IDENTITY, 1.0 - ARM_SHOULDER_INFLUENCE);

        //println!("quat align to target: {:?}", quat_align_to_target.to_euler(XYZ));

        let custom_pose = Transform::from_rotation(offset_quat * quat_align_to_target);
        let mut current_transform = local_transforms.get_mut(left_upper_arm_parent).unwrap();
        let rest_transform = bone_rests.get(left_upper_arm_parent).unwrap().0;

        current_transform.rotation = rest_transform.rotation * offset_quat * quat_align_to_target;

        let root = root * custom_pose;

        //println!("origin: {}", root.translation);
        //println!("rotation: {:?}", root.rotation.to_euler(XYZ));

        do_ik_bullshit(
            renik_limb.clone(),
            root,
            target,
            &bone_rests,
            skeleton,
            &mut local_transforms,
        );
    }
}

pub trait PoleOffset {
    fn pole_offset(&self) -> Quat;
}

pub fn left_arm_pole_offset() -> Quat {
    Quat::from_euler(
        EulerRot::XYZ,
        15.0f32.to_radians(),
        0.0,
        60.0f32.to_radians(),
    )
}

#[derive(Debug, Clone, Component, Reflect, InspectorOptions)]
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

fn do_ik_bullshit(
    limb: RenikLimb,
    root: Transform,
    local_target: Transform,
    bone_rests: &Query<&BoneRest>,
    skeleton: &HumanoidBones,
    local_transforms: &mut Query<&mut Transform>,
) {
    let true_root = root
        * Transform::from_translation(
            bone_rests
                .get(*skeleton.0.get(&BoneName::LeftUpperArm).unwrap())
                .unwrap()
                .0
                .translation,
        );

    let local_target = Transform::from_matrix(
        true_root.compute_matrix().inverse() * local_target.compute_matrix(),
    );

    let full_upper = bone_rests
        .get(*skeleton.0.get(&BoneName::LeftUpperArm).unwrap())
        .unwrap()
        .0;
    let full_lower = bone_rests
        .get(*skeleton.0.get(&BoneName::LeftLowerArm).unwrap())
        .unwrap()
        .0;
    let leaf = bone_rests
        .get(*skeleton.0.get(&BoneName::LeftHand).unwrap())
        .unwrap()
        .0;

    let upper_vector = full_lower.translation;
    let lower_vector = leaf.translation;

    let mut target_vector = local_target.translation;
    let normalized_target_vector = target_vector.normalize();

    let limb_length = upper_vector.length() + lower_vector.length();
    if target_vector.length() > upper_vector.length() + lower_vector.length() {
        target_vector = normalized_target_vector * limb_length;
    }

    let angles = trig_angles(upper_vector, lower_vector, target_vector);

    let starting_pole = limb.pole_offset * /*Vec3::new(0.0, 1.0, 0.0)*/ limb.a;
    let mut joint_axis = align_vectors(starting_pole, target_vector, 1.0)
        * (limb.pole_offset * /*Vec3::new(1.0, 0.0, 0.0)*/ limb.b);

    //println!("{}", joint_axis);

    let leaf_rest_vector = full_upper.rotation * (full_lower * leaf.translation);
    let positional_offset = limb
        .target_position_influence
        .dot(target_vector - leaf_rest_vector);
    joint_axis = Quat::from_axis_angle(
        normalized_target_vector,
        positional_offset + limb.roll_offset,
    )
    .mul_vec3(joint_axis);

    //println!("{}", joint_axis);
    let local_leaf_vector = local_target.rotation * (/*Vec3::new(0.0, 1.0, 0.0)*/limb.c);
    let local_lower_vector = Quat::from_axis_angle(joint_axis, angles.x - angles.y)
        .mul_vec3(normalized_target_vector)
        .normalize();

    let leaf_rejection = vector_rejection(local_leaf_vector, normalized_target_vector);
    let lower_rejection = vector_rejection(local_lower_vector, normalized_target_vector);

    let mut joint_roll_amount = if lower_rejection.length() == 0.0001 {
        leaf_rejection.angle_between(lower_rejection)
    } else {
        0.0
    } * limb.target_rotation_influence;
    joint_roll_amount *= local_leaf_vector
        .cross(local_lower_vector)
        .dot(normalized_target_vector)
        .abs();

    if leaf_rejection
        .cross(lower_rejection)
        .dot(normalized_target_vector)
        > 0.0
    {
        joint_roll_amount *= -1.0;
    }

    joint_axis =
        Quat::from_axis_angle(normalized_target_vector, joint_roll_amount).mul_vec3(joint_axis);

    //println!("{}", joint_axis);

    let total_roll = joint_roll_amount + positional_offset + limb.roll_offset;

    let leaf_x = align_vectors(
        Quat::from_axis_angle(normalized_target_vector, joint_roll_amount)
            .mul_vec3(local_leaf_vector),
        Quat::from_axis_angle(normalized_target_vector, joint_roll_amount)
            .mul_vec3(local_lower_vector),
        1.0,
    ) * (local_target.rotation * Vec3::new(1.0, 0.0, 0.0));
    let rolled_joint_axis =
        Quat::from_axis_angle(local_lower_vector, -1.0 * total_roll).mul_vec3(joint_axis);
    let lower_z = rolled_joint_axis.cross(local_lower_vector);
    let mut twist_angle = leaf_x.angle_between(rolled_joint_axis);
    if leaf_x.dot(lower_z) > 0.0 {
        twist_angle *= -1.0;
    }

    let mut inflection_point =
        if twist_angle > 0.0 { PI } else { -PI } - limb.twist_inflection_point_offset;
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
        unsafe { LEFT_ARM_OVERFLOW_STATE = 0.0 }
    }

    inflection_point += overflow_area;
    if twist_angle > 0.0 && twist_angle > inflection_point {
        twist_angle -= TAU;
    } else if twist_angle < 0.0 && twist_angle < inflection_point {
        twist_angle += TAU;
    }

    let mut lower_twist = twist_angle * limb.lower_limb_twist;
    let upper_twist = lower_twist * limb.upper_limb_twist + limb.upper_twist_offset - total_roll;

    lower_twist +=
        limb.lower_twist_offset - 2.0 * limb.roll_offset - positional_offset - joint_roll_amount;

    joint_axis = Quat::from_axis_angle(
        normalized_target_vector,
        twist_angle * limb.target_rotation_influence,
    )
    .mul_vec3(joint_axis);
    // Rebuild the rotations

    //println!("{}", joint_axis);

    let upper_joint_vector =
        Quat::from_axis_angle(joint_axis, angles.x).mul_vec3(normalized_target_vector);
    let rolled_lower_joint_axis =
        Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), -limb.roll_offset)
            .mul_vec3(Vec3::new(1.0, 0.0, 0.0));
    //println!("{}", rolled_lower_joint_axis);
    let lower_joint_vector =
        Quat::from_axis_angle(rolled_lower_joint_axis, angles.y).mul_vec3(Vec3::new(0.0, 1.0, 0.0));
    let twisted_joint_axis =
        Quat::from_axis_angle(upper_joint_vector, upper_twist).mul_vec3(joint_axis);
    let upper_basis = Quat::from_mat3(&Mat3::from_cols(
        twisted_joint_axis,
        upper_joint_vector,
        twisted_joint_axis.cross(upper_joint_vector),
    ));
    //println!("{:?}", lower_joint_vector);
    let mut lower_basis = Quat::from_mat3(&Mat3::from_cols(
        rolled_lower_joint_axis,
        lower_joint_vector,
        rolled_lower_joint_axis.cross(lower_joint_vector),
    ));

    //println!("{:?}", lower_basis.to_euler(XYZ));

    lower_basis = transpose_quat(lower_basis);

    //println!("{:?}", lower_basis.to_euler(XYZ));

    lower_basis *= Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), lower_twist);

    //println!("{:?}", lower_basis.to_euler(XYZ));

    //println!("{:?}", upper_twist);

    lower_basis *= Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), -upper_twist);

    //rotate_quat(lower_basis, Vec3::new(0.0, 1.0, 0.0), -upper_twist);
    //println!("{:?}", lower_basis.to_euler(XYZ));

    let upper_transform = (full_upper.rotation.inverse() * upper_basis).normalize();
    let lower_transform = (full_lower.rotation.inverse() * lower_basis).normalize();
    let leaf_transform = leaf.rotation.inverse()
        * (upper_basis * lower_basis).inverse()
        * local_target.rotation
        * leaf.rotation;

    /*println!("upper transform: {:?}", upper_transform.to_euler(XYZ));
    println!("lower transform: {:?}", lower_transform.to_euler(XYZ));
    println!("leaf_transform: {:?}", leaf_transform.to_euler(XYZ));*/

    local_transforms
        .get_mut(*skeleton.0.get(&BoneName::LeftUpperArm).unwrap())
        .unwrap()
        .rotation = upper_transform;
    local_transforms
        .get_mut(*skeleton.0.get(&BoneName::LeftLowerArm).unwrap())
        .unwrap()
        .rotation = lower_transform;
    local_transforms
        .get_mut(*skeleton.0.get(&BoneName::LeftHand).unwrap())
        .unwrap()
        .rotation = leaf_transform;
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

    let angle1 =
        safe_acos((length1_squared + length3_squared - length2_squared) / (length1 * length3));
    let angle2 =
        PI - safe_acos((length1_squared + length2_squared - length3_squared) / (length1 * length2));
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

        if perpendicular.length_squared() == 0.0 {
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
