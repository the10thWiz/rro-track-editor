use std::{
    fs::File,
    ops::{Add, Mul, Sub},
    time::{Duration, Instant}, path::PathBuf,
};

use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::Indices,
        primitives::{Frustum, Plane},
    },
};
use bevy_mod_picking::{Hover, PickingCamera, Selection};

use gvas::do_test;
use image::ImageFormat;
use smooth_bevy_cameras::controllers::orbit::{
    OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};

mod bevy_obj;
use bevy_obj::*;

// mod button;
mod background;
mod spline;
mod gvas;

// mod menu;
mod palette;
mod control;
mod snaps;
mod update;
// mod curve;
// mod spline_mesh;
// use spline_mesh::curve_offset;
// use button::{MouseAction, MouseOptions};
// use curve::{BSplineW, Bezier, CubicBezier, PolyBezier};
// use gvas::{RROSave, SplineType};

fn main() {
    do_test();
    App::new()
        .insert_resource(Msaa { samples: 4 })
        // .insert_resource(rro)
        // .insert_resource(path)
        // .insert_resource(BezierIDMax(0))
        .add_plugins(DefaultPlugins)
        .add_plugin(smooth_bevy_cameras::LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(WireframePlugin)
        .add_plugin(bevy_egui::EguiPlugin)
        .add_plugin(ObjPlugin)// Temp workaround to get bevy_obj to work
        .add_plugins(bevy_mod_picking::DefaultPickingPlugins)
        // .add_plugins(bevy_transform_gizmo::)
        // .add_plugin(button::Buttons)
        // .add_plugin(menu::MenuPlugin)
        .add_plugin(palette::PalettePlugin)
        .add_plugin(control::ControlPlugin)
        .add_plugin(background::Background)
        .add_startup_system(setup)
        // .add_startup_system(button_setup)
        // .add_system(button_test)
        // .add_startup_system(setup)
        // .add_system(transform_events)
        // .add_system(update_bezier)
        // .add_system(save)
        // .add_system(debugging)
        .run();
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
// pub enum MenuId {
//     Palette,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum MouseOpts {
//     LockZ
// }

// fn button_setup(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     // mut meshes: ResMut<Assets<Mesh>>,
//     // mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     let font = asset_server.load("fonts/FiraSans-Bold.ttf");
//     commands
//         .spawn_bundle(menu::MenuBundle::new(MenuId::Palette))
//         .with_children(|cmd| {
//             menu::option(cmd, &font, "Lock Z", MouseOpts::LockZ, true);
//         });
// }

// fn button_test(
//     bt: Query<&menu::Bool<MouseOpts>>,
// ) {
//     println!("Lock Z {}", menu::selected(&bt, &MouseOpts::LockZ));
// }

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle(DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 1000.,
                ..Default::default()
            },
            transform: Transform::from_rotation(Quat::from_rotation_x(0.8)),
            ..Default::default()
        });
    // camera
    commands
        .spawn_bundle(OrbitCameraBundle::new(
            OrbitCameraController {
                mouse_rotate_sensitivity: Vec2::splat(0.006),
                mouse_translate_sensitivity: Vec2::splat(0.08),
                mouse_wheel_zoom_sensitivity: 0.15,
                smoothing_weight: 0.0,
                enabled: true,
                pixels_per_line: 53.0,
            },
            PerspectiveCameraBundle::default(),
            Vec3::new(-2.0, 5.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
        ))
        .insert_bundle(bevy_mod_picking::PickingCameraBundle::default());
}