
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use bevy_mod_picking::PickingPluginsState;
use std::path::PathBuf;

use crate::gvas::SplineType;

/// File events for load and save
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileEvent {
    Load(PathBuf),
    Save(PathBuf),
}

/// Tool Palette State
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Palette {
    /// Current action
    pub action: MouseAction,
    /// Lock z axis
    pub lock_z: bool,
    /// Enable snapping
    pub snapping: bool,
    /// Show debug info
    pub show_debug: bool,
    /// Current file action
    file_action: FileAction,
}

/// Current file action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileAction {
    /// No active action
    None,
    /// Open file
    Open,
    /// Save file
    Save,
}

/// Current action when mouse is clicked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseAction {
    /// Drag individual control points
    Drag,
    /// Extend existing splines with new control points
    Extrude,
    /// TODO: Link existing splines end to end
    Link,
    /// Delete points or sections
    Delete,
    /// Place new splines
    Place,
    /// Toggle visibility of individual sections
    ToggleVisibility,
    /// Set the spline type of given spline
    SetSplineType(SplineType),
}

const SPLINE_TYPES: [(SplineType, &str); 5] = [
    (SplineType::Track, "Set Track"),
    (SplineType::TrackBed, "Set Track Bed"),
    (SplineType::GroundWork, "Set GroundWork"),
    (SplineType::WoodBridge, "Set Wood Bridge"),
    (SplineType::SteelBridge, "Set Steel Bridge"),
];

/// Plugin for the tool palette
pub struct PalettePlugin;

impl Plugin for PalettePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Palette {
            action: MouseAction::Drag,
            file_action: FileAction::None,
            lock_z: true,
            show_debug: cfg!(debug_assertions),
            snapping: false,
        });
        app.add_system(egui_system);
        app.add_event::<FileEvent>();
        app.insert_resource(DebugInfo::default());
    }
}

/// Debug info to show in the debug window
#[derive(Debug, Default, Clone, PartialEq, Hash)]
pub struct DebugInfo {
    /// Info for hovered object
    pub hovered: String,
}

fn egui_system(
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<Palette>,
    mut file_events: EventWriter<FileEvent>,
    debug_info: Res<DebugInfo>,
    mut picking_state: ResMut<PickingPluginsState>,
) {
    let state = state.as_mut();
    egui::Window::new("Palette")
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.label("File");
            if ui.button("Open").clicked() {
                state.file_action = FileAction::Open;
            }
            if ui.button("Save").clicked() {
                state.file_action = FileAction::Save;
            }
            ui.label("Actions");
            ui.radio_value(&mut state.action, MouseAction::Drag, "Drag");
            ui.radio_value(&mut state.action, MouseAction::Extrude, "Extrude");
            ui.radio_value(&mut state.action, MouseAction::Link, "Link(WIP)");
            ui.radio_value(&mut state.action, MouseAction::Delete, "Delete");
            ui.radio_value(&mut state.action, MouseAction::Place, "Place(WIP)");
            ui.radio_value(&mut state.action, MouseAction::ToggleVisibility, "ToggleVisibility");
            for (ty, text) in SPLINE_TYPES {
                ui.radio_value(&mut state.action, MouseAction::SetSplineType(ty), text);
            }
            ui.label("Options");
            ui.checkbox(&mut state.lock_z, "Lock Z");
            ui.checkbox(&mut state.show_debug, "Show Debug Info");
            ui.checkbox(&mut state.snapping, "Snapping(WIP)");
        });
    if matches!(state.file_action, FileAction::Open | FileAction::Save) {
        egui::Window::new("File")
            .resizable(false)
            .show(egui_context.ctx_mut(), |ui| {
                if let Some(save) = if ui.button("Slot 1").clicked() {
                    Some("slot1.sav")
                } else if ui.button("Slot 2").clicked() {
                    Some("slot2.sav")
                } else if ui.button("Slot 3").clicked() {
                    Some("slot3.sav")
                } else if ui.button("Slot 4").clicked() {
                    Some("slot4.sav")
                } else if ui.button("Slot 5").clicked() {
                    Some("slot5.sav")
                } else if ui.button("Slot 6").clicked() {
                    Some("slot6.sav")
                } else if ui.button("Slot 7").clicked() {
                    Some("slot7.sav")
                } else if ui.button("Slot 8").clicked() {
                    Some("slot8.sav")
                } else if ui.button("Slot 9").clicked() {
                    Some("slot9.sav")
                } else if ui.button("Slot 10").clicked() {
                    Some("slot10.sav")
                } else {
                    None
                } {
                    // println!("Action: {}", save);
                    // let path = PathBuf::from(std::env::var("LOCALAPPDATA"));
                    // println!("{:?}", std::env::var("LOCALAPPDATA"));
                    // let path: PathBuf = [
                    //     "c:\\",
                    //     "Users",
                    //     "PomesMatthew",
                    //     "AppData",
                    //     "Local",
                    //     "arr",
                    //     "Saved",
                    //     "SaveGames",
                    //     save
                    // ].iter().collect();
                    let path = PathBuf::from(std::env::var("LOCALAPPDATA").expect("Could not find local appdata"))
                        .join("arr")
                        .join("Saved")
                        .join("SaveGames")
                        .join(save);
                    match state.file_action {
                        FileAction::Open => file_events.send(FileEvent::Load(path)),
                        FileAction::Save => file_events.send(FileEvent::Save(path)),
                        _ => unreachable!(),
                    }
                    state.file_action = FileAction::None;
                }
            });
    }
    if state.show_debug {
        egui::Window::new("Debugging Info")
            .resizable(false)
            .show(egui_context.ctx_mut(), |ui| {
                ui.label("Hovered object:");
                ui.code(&debug_info.hovered);
            });
    }
}
