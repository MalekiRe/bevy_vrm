use bevy::prelude::*;
use bevy::transform;
use bevy_vrm::{HumanoidBones, SpringBones};

use crate::Settings;

pub fn draw_bones(
    mut gizmos: Gizmos,
    humanoid_bones: Query<&HumanoidBones>,
    transforms: Query<&GlobalTransform>,
    settings: Res<Settings>,
    parents: Query<&Parent>,
    children: Query<&Children>,
) {
    if !settings.draw_spring_bones {
        return;
    }

    for bones in humanoid_bones.iter() {
        for bone in bones.0.values() {
            let transform = transforms.get(*bone).unwrap().compute_transform();
            gizmos.sphere(transform.translation, Quat::default(), 0.01, Color::AZURE);
        }
    }
}

pub fn draw_spring_bones(
    mut gizmos: Gizmos,
    spring_bones: Query<&SpringBones>,
    transforms: Query<&GlobalTransform>,
    settings: Res<Settings>,
) {
    if !settings.draw_spring_bones {
        return;
    }

    for spring_bones in spring_bones.iter() {
        for spring_bone in spring_bones.0.iter() {
            for bone_entity in spring_bone.bones.iter() {
                let transform = match transforms.get(*bone_entity) {
                    Ok(t) => t,
                    Err(_) => {
                        continue;
                    }
                };

                gizmos.sphere(
                    transform.translation(),
                    Quat::default(),
                    spring_bone.hit_radius + 0.01,
                    Color::rgb(spring_bone.stiffness, 1.0 - spring_bone.stiffness, 0.1),
                );
            }
        }
    }
}
