use std::{
    fs::File,
    ops::{Add, Mul, Sub},
    time::{Duration, Instant}, path::PathBuf,
};

mod button;
mod curve;
mod gvas;

use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::Indices,
        primitives::{Frustum, Plane},
    },
};
use bevy_egui::EguiPlugin;
use bevy_mod_picking::{Hover, PickingCamera, Selection};
use bevy_transform_gizmo::{TransformGizmo, TransformGizmoEvent};
use button::{MouseAction, MouseOptions};
use curve::{mesh_from_curve, BSplineW, Bezier, CubicBezier, PolyBezier};
use gvas::{RROSave, SplineType};
use image::ImageFormat;
use smooth_bevy_cameras::controllers::orbit::{
    OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};

mod bevy_obj;
use bevy_obj::*;

// use nfd2::Response;

fn main() {
    // let file = nfd2::open_file_dialog(None, None).expect("Failed to open file");
    // let (rro, path) = match file {
    //     Response::Okay(path) => (
    //         RROSave::read(&mut File::open(&path).expect("Failed to open file"))
    //             .expect("Failed to read file"),
    //         path,
    //     ),
    //     Response::OkayMultiple(paths) => panic!("{:?}", paths),
    //     Response::Cancel => panic!("User Cancelled"),
    // };
    // println!("{:?}", std::env::current_dir().unwrap().read_dir().unwrap().collect::<Vec<_>>());
    println!("Started");
    // let path: PathBuf = ["c:\\", "Users", "PomesMatthew", "Documents", "rro-track-editor", "slot10.sav"].iter().collect();
    let path: PathBuf = ["c:\\", "Users", "PomesMatthew", "AppData", "Local", "arr", "Saved", "SaveGames", "slot10.sav"].iter().collect();
    // let path: PathBuf = PathBuf::new();
    println!("Created path");
    println!("Path: {}, {}", path.display(), path.exists());
    // 
    let rro = RROSave::read(&mut File::open(&path).expect("Failed to open file"))
                .expect("Failed to read file");
    println!("read file");
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(rro)
        .insert_resource(path)
        .insert_resource(BezierIDMax(0))
        .add_plugins(DefaultPlugins)
        .add_plugin(smooth_bevy_cameras::LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(WireframePlugin)
        .add_plugin(ObjPlugin)// Temp workaround to get bevy_obj to work
        .add_plugin(EguiPlugin)
        .add_plugins(bevy_mod_picking::DefaultPickingPlugins)
        .add_plugin(bevy_transform_gizmo::TransformGizmoPlugin::default()) // Use TransformGizmoPlugin::default() to align to the scene's coordinate system.
        .add_startup_system(setup)
        .add_startup_system(button::button_setup)
        .add_startup_system(load_height_map)
        .add_system(transform_events)
        .add_system(update_bezier)
        .add_system(save)
        .add_system(debugging)
        .add_system(button::button_system)
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
pub struct BezierHandle(usize, pub PolyBezier<CubicBezier>, SplineType);

#[derive(Debug, Component)]
pub struct BezierSection(usize, pub Handle<Mesh>);

pub struct BezierIDMax(usize);

pub const STEP: f32 = 0.1;
pub const ERR: f32 = 0.01;

fn transform_events(
    mut commands: Commands,
    pick_cam: Query<&PickingCamera>,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut objects: Query<(&mut DragState, &Hover, &mut Transform)>,
    mut beziers: Query<&mut BezierHandle>,
    mouse_opts: Res<MouseOptions>,
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
            let (state, sel, trans): (&mut DragState, &Hover, &mut Transform) =
                (state.as_mut(), sel, trans.as_mut());
            if sel.hovered() {
                state.initial = Some(trans.clone());
                state.drag_start = Some((trans.translation, picking_ray.direction()));
            }
        }
        if mouse_opts.action == MouseAction::Extrude {
            if let Some((s, transform)) = objects
                .iter()
                .find_map(|(s, _, _)| s.initial.map(|i| (s, i)))
            {
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
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 3. })),
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
    } else if mouse_button_input.just_released(MouseButton::Left) {
        for (mut state, sel, mut trans) in objects.iter_mut() {
            let (state, sel, trans): (&mut DragState, &Hover, &mut Transform) =
                (state.as_mut(), sel, trans.as_mut());
            state.initial = None;
            state.drag_start = None;
        }
    }

    for (mut state, sel, mut trans) in objects.iter_mut() {
        let (state, sel, trans): (&mut DragState, &Hover, &mut Transform) =
            (state.as_mut(), sel, trans.as_mut());
        if let Some((origin, dir)) = state.drag_start {
            let dir = if mouse_opts.lock_z {
                Vec3::new(0., 1., 0.)
            } else {
                dir
            };
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

fn save(
    keyboard: Res<Input<KeyCode>>,
    beziers: Query<&BezierHandle>,
    mut save_file: ResMut<RROSave>,
    save_path: Res<PathBuf>,
) {
    use gvas::CurveDataOwned;
    if keyboard.just_pressed(KeyCode::S)
        && (keyboard.pressed(KeyCode::RControl) || keyboard.pressed(KeyCode::LControl))
    {
        println!("Saving file to {}", save_path.display());
        save_file.set_curves(beziers.iter().map(|BezierHandle(_id, curve, ty)| {
            let pts: Vec<_> = curve.get_control_points().into_iter().map(|v| [v.z * 1000., v.x * 1000., v.y * 1000.]).collect();
            CurveDataOwned {
                location: pts[0],
                ty: *ty,
                visibility: vec![true; pts.len() - 1],
                control_points: pts,
            }
        })).expect("Failed to update file");
        save_file.write(&mut File::create(save_path.as_path()).unwrap()).unwrap();
    }
}

fn debugging(
    keyboard: Res<Input<KeyCode>>,
    beziers: Query<&BezierHandle>,
    control_points: Query<(&DragState, &Hover)>,
    curve_segments: Query<(&BezierSection, &Hover)>,
    other: Query<&Transform, With<bevy_transform_gizmo::GizmoTransformable>>,
) {
    if keyboard.just_pressed(KeyCode::D) {
        let mut id = None;
        for (dr, hover) in control_points.iter() {
            if hover.hovered() {
                id = Some(dr.id);
            }
        }
        for (dr, hover) in curve_segments.iter() {
            if hover.hovered() {
                id = Some(dr.0);
            }
        }
        if let Some(id) = id {
            for BezierHandle(_id, curve, ty) in beziers.iter() {
                if *_id == id {
                    println!("Curve: {:?}", curve);
                    println!("\ttype: {:?}", ty);
                }
            }
        }
        for tr in other.iter() {
            println!("{:?}", tr);
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
    let start = Instant::now();
    for mut b in beziers.iter_mut() {
        //println!("Spawning bezier");
        for mesh in b.1.create_meshes(meshes.as_mut()) {
            let section = BezierSection(b.0, mesh.clone_weak());
            commands
                .spawn_bundle(PbrBundle {
                    mesh,
                    material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
                    ..Default::default()
                })
                .insert_bundle(bevy_mod_picking::PickableBundle::default())
                .insert(section);
            //.insert(Wireframe);
            //println!("Spawning bezier: {}", b.0);
        }
        b.1.update_transforms(sections.iter_mut().filter(|(_, s)| s.0 == b.0));
        // Allow partial rendering
        if start.elapsed() > Duration::from_millis(30) {
            break;
        }
        //println!("Spawning bezier");
    }
    //println!("Done");
}

fn spawn_bezier(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    id: usize,
    pts: Vec<Vec3>,
) -> PolyBezier<CubicBezier> {
    for (i, &pt) in pts.iter().enumerate() {
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.3 })),
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
    PolyBezier::new(pts)
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    save_file: Res<RROSave>,
    mut max_id: ResMut<BezierIDMax>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
   
    for (i, curve) in save_file.curves().expect("Save File Format").enumerate() {
        let pts: Vec<_> = curve.control_points.iter().map(|&[a, b, c]| Vec3::new(b / 1000., c / 1000., a / 1000.)).collect();
        let bezier = spawn_bezier(&mut commands, &mut meshes, &mut materials, i, pts);
        commands.spawn().insert(BezierHandle(i, bezier, curve.ty));
        max_id.0 += 1;
    }

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
        .insert_bundle(bevy_mod_picking::PickingCameraBundle::default())
        .insert(bevy_transform_gizmo::GizmoPickSource::default());
}

fn load_height_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
    .spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10. })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        ..Default::default()
    });
    // commands
    //     .spawn_bundle(PbrBundle {
    //         mesh: asset_server.load("rro_height_map.obj"),
    //         material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
    //         transform: Transform::from_rotation(Quat::from_rotation_y(-std::f32::consts::PI/2.)).with_scale(Vec3::new(4.8, 4.8, 4.8)),
    //         ..Default::default()
    //     })
    //     .insert_bundle(bevy_mod_picking::PickableBundle::default())
    //     .insert(bevy_transform_gizmo::GizmoTransformable);
}
