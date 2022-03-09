
use bevy::prelude::*;

pub struct Background;

impl Plugin for Background {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_height_map);
    }
}

fn load_height_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 100. })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            ..Default::default()
        });
    // commands
    //     .spawn_bundle(PbrBundle {
    //         mesh: asset_server.load("rro_height_map.obj"),
    //         material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
    //         transform: Transform::from_rotation(Quat::from_rotation_y(-std::f32::consts::PI/2.))
    //                                    .with_scale(Vec3::new(4.8, 4.8, 4.8)),
    //         ..Default::default()
    //     });
}
