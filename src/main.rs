use bevy::{pbr::wireframe::WireframePlugin, prelude::*};
use smooth_bevy_cameras::controllers::orbit::{
    OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};

mod bevy_obj;

mod background;
mod gvas;
mod spline;

mod control;
mod palette;
mod snaps;
mod update;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(smooth_bevy_cameras::LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(WireframePlugin)
        .add_plugin(bevy_egui::EguiPlugin)
        .add_plugin(bevy_obj::ObjPlugin) // Temp workaround to get bevy_obj to work
        .add_plugin(bevy_mod_picking::PickingPlugin)
        .add_plugin(bevy_mod_picking::InteractablePickingPlugin)
        .add_plugin(bevy_mod_picking::HighlightablePickingPlugin)
        .add_plugin(palette::PalettePlugin)
        .add_plugin(control::ControlPlugin)
        .add_plugin(background::Background)
        .add_plugin(snaps::SnapPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(DirectionalLightBundle {
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
