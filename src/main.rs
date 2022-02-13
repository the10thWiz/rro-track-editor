use std::ops::{Add, Mul, Sub};

mod curve;
mod gvas;

use bevy::{prelude::*, reflect::TypeUuid, render::mesh::Indices, pbr::wireframe::{Wireframe, WireframePlugin}};
use bevy_mod_picking::{PickingCamera, Selection};
use bevy_transform_gizmo::{TransformGizmo, TransformGizmoEvent};
use curve::{mesh_from_curve, BSplineW, Bezier, CubicBezier, PolyBezier};
use smooth_bevy_cameras::controllers::orbit::{
    OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(smooth_bevy_cameras::LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(WireframePlugin)
        .add_plugins(bevy_mod_picking::DefaultPickingPlugins)
        .add_plugin(bevy_transform_gizmo::TransformGizmoPlugin::new(
            Quat::from_rotation_y(-0.2), // Align the gizmo to a different coordinate system.
        )) // Use TransformGizmoPlugin::default() to align to the scene's coordinate system.
        .add_startup_system(setup)
        .add_system(transform_events)
        .add_system(update_bezier)
        .run();
}

#[derive(Debug, Component, Default)]
pub struct DragState {
    id: usize,
    pt: usize,
    drag_start: Option<(Vec3, Vec3)>,
    initial: Option<Transform>,
}

#[derive(Debug, Component)]
pub struct BezierHandle(usize, pub PolyBezier<CubicBezier>);

#[derive(Debug, Component)]
pub struct BezierSection(usize, pub Handle<Mesh>);

pub const STEP: f32 = 0.1;
pub const ERR: f32 = 0.01;

fn transform_events(
    mut commands: Commands,
    pick_cam: Query<&PickingCamera>,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut objects: Query<(&mut DragState, &Selection, &mut Transform)>,
    mut beziers: Query<&mut BezierHandle>,
) {
    let picking_camera: &PickingCamera = if let Some(cam) = pick_cam.iter().last() {
        cam
    } else {
        error!("Not exactly one picking camera.");
        return;
    };
    let picking_ray = if let Some(ray) = picking_camera.ray() {
        ray
    } else {
        error!("Picking camera does not have a ray.");
        return;
    };
    if mouse_button_input.just_pressed(MouseButton::Left) {
        for (mut state, sel, mut trans) in objects.iter_mut() {
            let (state, sel, trans): (&mut DragState, &Selection, &mut Transform) =
                (state.as_mut(), sel, trans.as_mut());
            if sel.selected() {
                state.initial = Some(trans.clone());
                state.drag_start = Some((trans.translation, picking_ray.direction()));
            }
        }
    } else if mouse_button_input.just_released(MouseButton::Left) {
        for (mut state, sel, mut trans) in objects.iter_mut() {
            let (state, sel, trans): (&mut DragState, &Selection, &mut Transform) =
                (state.as_mut(), sel, trans.as_mut());
            state.initial = None;
            state.drag_start = None;
        }
    }

    if mouse_button_input.pressed(MouseButton::Left) && keyboard.just_released(KeyCode::E) {
        if let Some((s, transform)) = objects.iter().find_map(|(s, _, _)| s.initial.map(|i| (s, i))) {
            let id = s.id;
            let pt = s.pt;
            for (mut s, _, _) in objects.iter_mut() {
                if s.id == id && s.pt > pt {
                    s.pt += 1;
                }
            }
            for mut handle in beziers.iter_mut() {
                if handle.0 == id {
                    handle.1.insert(pt, transform.translation);
                }
            }
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform,
                    ..Default::default()
                })
                .insert_bundle(bevy_mod_picking::PickableBundle::default())
                .insert(DragState {
                    id,
                    pt: pt + 1,
                    ..DragState::default()
                });
        }
    }
    for (mut state, sel, mut trans) in objects.iter_mut() {
        let (state, sel, trans): (&mut DragState, &Selection, &mut Transform) =
            (state.as_mut(), sel, trans.as_mut());
        if let Some((origin, dir)) = state.drag_start {
            if let Some(int) =
                picking_camera.intersect_primitive(bevy_mod_picking::Primitive3d::Plane {
                    point: origin,
                    normal: dir,
                })
            {
                let dir = int.position() - origin;
                let mut init = match state.initial {
                    Some(initial) => initial,
                    None => unreachable!(),
                };
                init.translation += dir;
                if keyboard.just_pressed(KeyCode::D) {
                    dbg!(&state);
                    dbg!(&init);
                }
                *trans = init;
                if let Some(mut b) = beziers.iter_mut().find(|b| b.0 == state.id) {
                    //b.1.translate(state.pt, init.translation);
                    b.1.update(state.pt, init.translation);
                }
            }
        }
    }
}

fn update_bezier(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut beziers: Query<&mut BezierHandle>,
    mut sections: Query<(&mut Transform, &BezierSection)>,
) {
    for mut b in beziers.iter_mut() {
        for mesh in b.1.create_meshes(meshes.as_mut()) {
            let section = BezierSection(b.0, mesh.clone_weak());
            commands.spawn_bundle(PbrBundle {
                mesh,
                material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
                ..Default::default()
            }).insert(section).insert(Wireframe);
        }
        b.1.update_transforms(sections.iter_mut().filter(|(_, s)| s.0 == b.0));
    }
}

fn spawn_bezier(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    id: usize,
    pts: [Vec3; 4],
) {
    let curve = PolyBezier::new(vec![pts[0], pts[1], pts[2], pts[3]]);
    //let mesh = mesh_from_curve(curve.walker(STEP, ERR));
    //let mesh = mesh_from_curve(BSplineW::new(vec![
    //pts[0], pts[1], pts[2], pts[3],
    //]).walker(STEP));
    //let bezier = meshes.add(mesh);
    //for mesh in curve.create_meshes(meshes) {
        //let section = BezierSection(id, mesh.clone_weak());
        //commands.spawn_bundle(PbrBundle {
            //mesh,
            //material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
            //..Default::default()
        //}).insert(section).insert(Wireframe);
    //}
    commands.spawn().insert(BezierHandle(id, curve));
    for (i, &pt) in pts.iter().enumerate() {
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_translation(pt),
                ..Default::default()
            })
            .insert_bundle(bevy_mod_picking::PickableBundle::default())
            .insert(DragState {
                id,
                pt: i,
                ..DragState::default()
            });
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    //.insert_bundle(bevy_mod_picking::PickableBundle::default())
    //.insert(bevy_transform_gizmo::GizmoTransformable);
    //let m = meshes.get_mut(todo!()).unwrap();
    //// cube
    //let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleStrip);
    //mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vec![
    //[1., 0.5, 0.],
    //[1., 0.5, 1.],
    //[0., 0.5, 1.],
    //]);
    //mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vec![
    //[0., 1., 0.],
    //[0., 1., 0.],
    //[0., 1., 0.],
    //]);
    //mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![
    //[0., 1.],
    //[1., 1.],
    //[1., 0.],
    //]);
    //mesh.set_indices(Some(Indices::U32(vec![0, 2, 1])));
    spawn_bezier(
        &mut commands,
        &mut meshes,
        &mut materials,
        0,
        [
            Vec3::new(1., 0.5, 1.),
            Vec3::new(-1., 0.5, 1.),
            Vec3::new(-1., 0.5, -1.),
            Vec3::new(1., 0.5, -1.),
        ],
    );
    //spawn_bezier(
        //&mut commands,
        //&mut meshes,
        //&mut materials,
        //1,
        //[
            //Vec3::new(1., 0.0, 1.),
            //Vec3::new(-1., 0.0, 1.),
            //Vec3::new(-1., 0.0, -1.),
            //Vec3::new(1., 0.0, -1.),
        //],
    //);

    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands
        .spawn_bundle(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            PerspectiveCameraBundle::default(),
            Vec3::new(-2.0, 5.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
        ))
        .insert_bundle(bevy_mod_picking::PickingCameraBundle::default())
        .insert(bevy_transform_gizmo::GizmoPickSource::default());
}
