//! Drag and drop [VRM](https://vrm.dev/) viewer using [bevy_vrm](https://github.com/unavi-xyz/bevy_vrm).

use std::f32::consts::PI;

use bevy::prelude::*;

use bevy::transform::TransformSystem::TransformPropagate;
use bevy_egui::EguiPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_vrm::{loader::Vrm, mtoon::MtoonSun, SpringBones, VrmBundle, VrmPlugin};

mod draw_spring_bones;
mod move_leg;
mod ui;

pub struct VrmViewerPlugin;

impl Plugin for VrmViewerPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_family = "wasm")]
        {
            app.add_plugins(bevy_web_file_drop::WebFileDropPlugin);
        }

        app.insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
            .init_resource::<Settings>()
            .add_plugins((DefaultPlugins, EguiPlugin, PanOrbitCameraPlugin, VrmPlugin))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    draw_spring_bones::draw_spring_bones,
                    draw_spring_bones::draw_bones,
                    move_leg::move_leg,
                    read_dropped_files,
                    ui::update_ui,
                    move_avatar,
                ),
            )
            .add_systems(
                Update,
                (add_recursive_spring_bones, add_springbone_logic_state).chain(),
            )
            .add_systems(PostUpdate, (do_springbone_logic).after(TransformPropagate));
    }
}

fn move_avatar(mut query: Query<&mut Transform, With<SpringBones>>, time: Res<Time>) {
    let move_speed = (time.elapsed_seconds() + 1.0) / 10.0;
    for mut t in query.iter_mut() {
        let a = time.elapsed_seconds() * move_speed;
        let b = a.sin();
        t.rotation.x = b / 20.0;
        t.translation.x += b / 70.0;
    }
}

#[derive(Component)]
pub struct SpringBoneLogicState {
    prev_tail: Vec3,
    current_tail: Vec3,
    pub bone_axis: Vec3,
    pub bone_length: f32,
    initial_local_matrix: Mat4,
    initial_local_rotation: Quat,
}

fn add_recursive_spring_bones(
    mut spring_boness: Query<&mut SpringBones>,
    children: Query<&Children>,
) {
    for mut spring_bones in spring_boness.iter_mut() {
        for spring_bone in spring_bones.0.iter_mut() {
            let bones = spring_bone.bones.clone();
            for bone in bones {
                for child in children.iter_descendants(bone) {
                    if !spring_bone.bones.contains(&child) {
                        spring_bone.bones.push(child);
                    }
                }
            }
        }
    }
}

fn add_springbone_logic_state(
    mut commands: Commands,
    spring_boness: Query<(Entity, &SpringBones)>,
    logic_states: Query<&mut SpringBoneLogicState>,
    global_transforms: Query<&GlobalTransform>,
    local_transforms: Query<&Transform>,
    children: Query<&Children>,
    names: Query<&Name>,
) {
    for (_skel_e, spring_bones) in spring_boness.iter() {
        for spring_bone in spring_bones.0.iter() {
            for (_i, bone) in spring_bone.bones.iter().enumerate() {
                /*if let Ok(name) = names.get(*bone) {
                    println!("{}", name);
                }*/
                if !logic_states.contains(*bone) {
                    /*if let Ok(name) = names.get(*bone) {
                        if name.as_str() != "donotaddmore" {
                            println!("{}", name);
                        }
                    }*/
                    let child = match children.get(*bone) {
                        Ok(c) => c,
                        Err(_) => {
                            if let Ok(name) = names.get(*bone) {
                                if name.as_str() == "donotaddmore" {
                                    continue;
                                }
                            }
                            let child = commands
                                .spawn((
                                    TransformBundle {
                                        local: Transform::from_xyz(0.0, -0.07, 0.0),
                                        global: Default::default(),
                                    },
                                    Name::new("donotaddmore"),
                                ))
                                .id();

                            commands.entity(*bone).add_child(child);
                            continue;
                        }
                    };
                    let mut next_bone = None;
                    for c in child.iter() {
                        next_bone.replace(*c);
                        break;
                    }
                    let next_bone = match next_bone {
                        None => continue,
                        Some(next_bone) => next_bone,
                    };

                    let global_next_bone = global_transforms.get(*bone).unwrap();

                    let local_next_bone = local_transforms.get(next_bone).unwrap();

                    let local_this_bone = local_transforms.get(*bone).unwrap();

                    let bone_axis = local_next_bone.translation.normalize();

                    let bone_length = local_next_bone.translation.length();

                    let initial_local_matrix = local_this_bone.compute_matrix();
                    let initial_local_rotation = local_this_bone.rotation;

                    commands.entity(*bone).insert(SpringBoneLogicState {
                        prev_tail: global_next_bone.translation(),
                        current_tail: global_next_bone.translation(),
                        bone_axis,
                        bone_length,
                        initial_local_matrix,
                        initial_local_rotation,
                    });
                }
            }
        }
    }
}

fn do_springbone_logic(
    mut global_transforms: Query<(&mut GlobalTransform, &mut Transform)>,
    spring_boness: Query<&SpringBones>,
    mut spring_bone_logic_states: Query<&mut SpringBoneLogicState>,
    parents: Query<&Parent>,
    time: Res<Time>,
    names: Query<&Name>,
) {
    for spring_bones in spring_boness.iter() {
        for spring_bone in spring_bones.0.iter() {
            println!();
            for bone in spring_bone.bones.iter() {
                if let Ok(name) = names.get(*bone) {
                    println!("{}", name);
                }
            }
            println!();

            for (_i, bone) in spring_bone.bones.iter().enumerate() {
                let bone: Entity = *bone;
                let (global, _) = global_transforms.get(bone).unwrap();
                let mut spring_bone_logic_state = match spring_bone_logic_states.get_mut(bone) {
                    Ok(spring_bone_logic_state) => spring_bone_logic_state,
                    Err(_) => continue,
                };
                let world_position = *global;

                let parent_entity = parents.get(bone).unwrap().get();

                let parent_world_rotation = global_transforms
                    .get(parent_entity)
                    .unwrap()
                    .0
                    .to_scale_rotation_translation()
                    .1;

                let inertia = (spring_bone_logic_state.current_tail
                    - spring_bone_logic_state.prev_tail)
                    * (1.0 - spring_bone.drag_force);
                let stiffness = time.delta_seconds()
                    * (parent_world_rotation
                        * spring_bone_logic_state.bone_axis
                        * spring_bone.stiffness);
                let external =
                    time.delta_seconds() * spring_bone.gravity_dir * spring_bone.gravity_power;

                let mut next_tail =
                    spring_bone_logic_state.current_tail + inertia + stiffness + external;

                if let Ok(name) = names.get(bone) {
                    println!("{name}, {}", next_tail);
                }

                next_tail = world_position.translation()
                    + (next_tail - world_position.translation()).normalize()
                        * spring_bone_logic_state.bone_length;

                spring_bone_logic_state.prev_tail = spring_bone_logic_state.current_tail;
                spring_bone_logic_state.current_tail = next_tail;

                let parent_world_matrix = global_transforms
                    .get(parent_entity)
                    .unwrap()
                    .0
                    .compute_matrix();

                let parent_pos = *global_transforms.get(parent_entity).unwrap().0;

                let to = ((parent_world_matrix * spring_bone_logic_state.initial_local_matrix)
                    .inverse()
                    .transform_point3(next_tail))
                .normalize();

                let (mut global, mut local) = global_transforms.get_mut(bone).unwrap();

                local.rotation = spring_bone_logic_state.initial_local_rotation
                    * Quat::from_rotation_arc(spring_bone_logic_state.bone_axis, to);

                *global = parent_pos.mul_transform(*local);
            }
        }
    }
}

#[derive(Resource, Default)]
struct Settings {
    pub draw_spring_bones: bool,
    pub move_leg: bool,
}

#[cfg(target_family = "wasm")]
const VRM_PATH: &str = "/bevy_vrm/assets/suzuha.vrm";
#[cfg(not(target_family = "wasm"))]
const VRM_PATH_2: &str = "AliciaSolid_vrm-0.51.vrm";
const VRM_PATH: &str = "suzuha.vrm";

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut config: ResMut<GizmoConfigStore>,
) {
    let (config, _) = config.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -1.0;
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(1.0, 2.0, 5.0),
            ..default()
        },
        PanOrbitCamera {
            focus: Vec3::new(0.0, 0.8, 0.0),
            ..default()
        },
    ));

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 10_000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_rotation(Quat::from_rotation_x(-PI / 3.0)),
            ..default()
        },
        MtoonSun,
    ));

    let mut transform = Transform::default();
    transform.rotate_y(PI);

    transform.translation.x -= 1.0;

    commands.spawn(VrmBundle {
        scene_bundle: SceneBundle {
            transform,
            ..default()
        },
        vrm: asset_server.load(VRM_PATH),
        ..default()
    });

    transform.translation.x -= 1.0;

    commands.spawn(VrmBundle {
        scene_bundle: SceneBundle {
            transform,
            ..default()
        },
        vrm: asset_server.load(VRM_PATH_2),
        ..default()
    });
}

fn read_dropped_files(
    mut commands: Commands,
    mut events: EventReader<FileDragAndDrop>,
    asset_server: Res<AssetServer>,
    mut vrms: Query<Entity, With<Handle<Vrm>>>,
) {
    for event in events.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            #[cfg(target_family = "wasm")]
            let path = String::from(path_buf.to_str().unwrap());
            #[cfg(not(target_family = "wasm"))]
            let path = bevy::asset::AssetPath::from_path(path_buf.as_path());

            info!("DroppedFile: {}", path);

            let entity = vrms.single_mut();
            commands.entity(entity).despawn_recursive();

            let mut transform = Transform::default();
            transform.rotate_y(PI);

            commands.spawn(VrmBundle {
                scene_bundle: SceneBundle {
                    transform,
                    ..default()
                },
                vrm: asset_server.load(path),
                ..default()
            });
        }
    }
}
