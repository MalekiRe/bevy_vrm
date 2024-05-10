use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::egui::{CollapsingHeader, ScrollArea};
use bevy_egui::EguiContext;

pub fn update_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    bevy_egui::egui::Window::new("UI").show(egui_context.get_mut(), |ui| {
        ScrollArea::vertical().show(ui, |ui| {
            // equivalent to `WorldInspectorPlugin`
            bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);

            CollapsingHeader::new("Materials").show(ui, |ui| {
                bevy_inspector_egui::bevy_inspector::ui_for_assets::<StandardMaterial>(world, ui);
            });

            ui.heading("Entities");
            bevy_inspector_egui::bevy_inspector::ui_for_world_entities(world, ui);
        });
    });
}
/*
pub fn update_ui(mut contexts: EguiContexts, mut settings: ResMut<Settings>) {
    bevy_egui::egui::Window::new("VRM Viewer").show(contexts.ctx_mut(), |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                ui.label("Loads ");
                ui.hyperlink_to("VRM", "https://vrm.dev/en");
                ui.label(" avatars using ");
                ui.hyperlink_to("bevy_vrm", "https://github.com/unavi-xyz/bevy_vrm");
                ui.label(", a plugin for the ");
                ui.hyperlink_to("Bevy", "https://bevyengine.org");
                ui.label(" game engine.");
            });

            ui.label("Drop a .vrm file into the window to load it.");

            ui.separator();

            ui.checkbox(&mut settings.draw_spring_bones, "Draw spring bones");
            ui.checkbox(&mut settings.move_leg, "Move leg bone");

            ui.separator();

            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.hyperlink_to("[github]", "https://github.com/unavi-xyz/bevy_vrm");
                });
            });
        });
    });
}
*/
