use bevy::{input::common_conditions::input_toggle_active, prelude::KeyCode, prelude::*};
use bevy_egui::EguiContext;
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;
use bevy_window::PrimaryWindow;

//mod super::components;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (inspector_ui.run_if(input_toggle_active(true, KeyCode::Escape)),),
        );
    }
}

fn inspector_ui(world: &mut World, mut selected_entities: Local<SelectedEntities>) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();
    egui::SidePanel::left("hierarchy")
        .default_width(200.0)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Hierarchy");

                bevy_inspector_egui::bevy_inspector::hierarchy::hierarchy_ui(
                    world,
                    ui,
                    &mut selected_entities,
                );

                ui.label("Press escape to toggle UI");
                ui.allocate_space(ui.available_size());
            });
        });

    egui::SidePanel::right("inspector")
        .default_width(250.0)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Inspector");

                match selected_entities.as_slice() {
                    &[entity] => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entity(world, entity, ui);
                    }
                    entities => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entities_shared_components(
                            world, entities, ui,
                        );
                    }
                }

                ui.allocate_space(ui.available_size());
            });
        });
}

use crate::components::cards::CARD_SIZE;

pub fn gizmo_update(
    mut gizmos: Gizmos,
    mut giz: Query<(&mut Transform, &crate::components::cards::CardSlot)>,
) {
    for (t, _) in giz.iter_mut() {
        gizmos.rect(t.translation, t.rotation, CARD_SIZE, Color::RED);
    }
}

