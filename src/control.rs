use crate::gvas::{SplineType, RROSave, CurveDataOwned};
use crate::palette::{FileEvent, MouseAction, Palette};
use crate::spline::mesh::curve_offset;
use crate::spline::{CubicBezier, PolyBezier};
use bevy::prelude::*;
use bevy_mod_picking::{Hover, PickingCamera};
use std::fs::File;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use enum_map::{EnumMap, enum_map};

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_assets);
        app.insert_resource(RROSave::read(&mut std::io::Cursor::new(include_bytes!("../assets/slot10.sav"))).expect("Failed to parse included save"));
        app.add_event::<BezierModificaiton>();
        app.add_system(load_save);
        app.add_system(update_bezier_transform);
        app.add_system(update_curve_sections);
        app.add_system(modify_beziers);
        app.add_system(debugging);
    }
}

pub struct DefaultAssets {
    pub handle_mesh: Handle<Mesh>,
    pub handle_material: Handle<StandardMaterial>,
    pub spline_material: EnumMap<SplineType, Handle<StandardMaterial>>,
}

fn init_assets(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let handle_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.3 }));
    let handle_material = materials.add(Color::rgb(0.8, 0.0, 0.0).into());
    let mut spline_material = enum_map!{
            SplineType::GroundWork => materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            SplineType::ConstGroundWork => materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            SplineType::Track => materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            SplineType::TrackBed => materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            SplineType::WoodBridge => materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            SplineType::SteelBridge => materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            SplineType::StoneGroundWork => materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            SplineType::ConstStoneGroundWork => materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    };
    commands.insert_resource(DefaultAssets {
        handle_mesh,
        handle_material,
        spline_material,
    });
}

fn load_save(
    mut events: EventReader<FileEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    beziers: Query<(Entity, &PolyBezier<CubicBezier>)>,
    mut gvas: ResMut<RROSave>,
    mut commands: Commands,
) {
    for event in events.iter() {
        if let Err(e) = match event {
            FileEvent::Load(path) => load_file(path, &mut meshes, &beziers, &mut materials, &mut commands),
            FileEvent::Save(path) => save_file(path, &beziers, &mut gvas),
        } {
            println!("Error: {:?}", e);
        }
    }
}

/// The minimal set of components to create an empty parent for meshes
#[derive(Debug, Bundle, Default)]
struct ParentBundle {
    _local: Transform,
    _global: GlobalTransform,
}

fn save_file(
    path: &PathBuf,
    beziers: &Query<(Entity, &PolyBezier<CubicBezier>)>,
    gvas: &mut ResMut<RROSave>,
) -> Result<(), crate::gvas::GVASError> {
    gvas.set_curves(beziers.iter().map(|(_e, b)| {
        let control_points: Vec<_> = b.get_control_points().map(|v| [v.z * 1000., v.x * 1000., v.y * 1000.]).collect();
        CurveDataOwned {
            location: control_points[0],
            ty: b.ty(),
            visibility: vec![true; control_points.len() - 1],
            control_points,
        }
    }))?;
    gvas.write(&mut File::create(path)?)?;
    Ok(())
}

fn load_file(
    path: &PathBuf,
    meshes: &mut ResMut<Assets<Mesh>>,
    beziers: &Query<(Entity, &PolyBezier<CubicBezier>)>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    commands: &mut Commands,
) -> Result<(), crate::gvas::GVASError> {
    // Clear the world
    for (e, _p) in beziers.iter() {
        commands.entity(e).despawn_recursive();
    }
    // Load from file
    let gvas = crate::gvas::RROSave::read(&mut File::open(path)?)?;
    for curve in gvas.curves()? {
        // TODO: spawn curves
        let mut entity = commands.spawn_bundle(ParentBundle::default());
        let points: Vec<_> = curve
            .control_points
            .iter()
            .map(|&[a, b, c]| Vec3::new(b / 1000., c / 1000., a / 1000.))
            .collect();
        entity.with_children(|commands| {
            for (i, point) in points.iter().enumerate() {
                println!("Spawning point at {}", point);
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.3 })),
                        material: materials.add(Color::rgb(0.8, 0.0, 0.0).into()),
                        transform: Transform::from_translation(*point + curve_offset(curve.ty)),
                        ..Default::default()
                    })
                    .insert_bundle(bevy_mod_picking::PickableBundle::default())
                    .insert(DragState {
                        pt: i,
                        ..DragState::default()
                    });
            }
        });
        let bezier = PolyBezier::new(points, curve.ty);
        entity.insert(bezier);
    }
    commands.insert_resource(gvas);
    Ok(())
}

#[derive(Debug, Component, Default)]
struct DragState {
    pt: usize,
    drag_start: Option<(Vec3, Vec3)>,
    initial: Option<Transform>,
}

#[derive(Debug, Component, Default)]
struct BezierSection(Handle<Mesh>);

#[derive(Debug, Clone, PartialEq)]
enum BezierModificaiton {
    Extrude(Entity, usize),
    Delete(Entity, usize),
    Place(Vec3, Vec3),
    ChangeTy(Entity, SplineType, SplineType),
}

fn debugging(
    keyboard: Res<Input<KeyCode>>,
    objects: Query<(&Hover, &Transform, &Parent, &DragState)>,
    beziers: Query<&PolyBezier<CubicBezier>>,
) {
    if keyboard.just_pressed(KeyCode::D) {
        for (hover, trans, parent, state) in objects.iter() {
            if hover.hovered() {
                let bez = beziers.get(parent.0.clone()).unwrap();
                print!("Point: {}, ", trans.translation - curve_offset(bez.ty()));
                print!("ty: {:?}, ", bez.ty());
                print!("pt: {}, ", state.pt);
                println!();
            }
        }
    }
}

fn update_bezier_transform(
    pick_cam: Query<&PickingCamera>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut objects: Query<(&mut DragState, &Hover, &mut Transform, &Parent)>,
    mut beziers: Query<&mut PolyBezier<CubicBezier>>,
    mut palette: ResMut<Palette>,
    mut modification: EventWriter<BezierModificaiton>,
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
        if matches!(palette.action, MouseAction::Drag | MouseAction::Extrude) {
            for (mut state, hover, trans, parent) in objects.iter_mut() {
                if hover.hovered() {
                    state.initial = Some(trans.clone());
                    state.drag_start = Some((trans.translation, picking_ray.direction()));
                    if matches!(palette.action, MouseAction::Extrude) {
                        let mut bez = beziers.get_mut(parent.0.clone()).unwrap();
                        let loc = trans.translation - curve_offset(bez.ty());
                        bez.insert(state.pt, loc);
                        modification.send(BezierModificaiton::Extrude(parent.0.clone(), state.pt));
                        palette.action = MouseAction::Drag;
                    }
                }
            }
        } else if matches!(palette.action, MouseAction::Place) {
            modification.send(BezierModificaiton::Place(
                picking_ray.origin(),
                picking_ray.direction(),
            ));
        } else if matches!(palette.action, MouseAction::Delete) {
            for (state, hover, _trans, parent) in objects.iter() {
                if hover.hovered() {
                    modification.send(BezierModificaiton::Delete(parent.0.clone(), state.pt));
                    break;
                }
            }
        } else if let MouseAction::SetSplineType(ty) = palette.action {
            for (_state, hover, _trans, parent) in objects.iter() {
                if hover.hovered() {
                    let mut bez = beziers.get_mut(parent.0.clone()).unwrap();
                    modification.send(BezierModificaiton::ChangeTy(parent.0.clone(), bez.ty(), ty));
                    bez.set_ty(ty);
                    break;
                }
            }
        }
    } else if mouse_button_input.just_released(MouseButton::Left) {
        for (mut state, _sel, _trans, _) in objects.iter_mut() {
            state.initial = None;
            state.drag_start = None;
        }
    }

    for (state, _sel, mut trans, parent) in objects.iter_mut() {
        if let Some((origin, dir)) = state.drag_start {
            let dir = if palette.lock_z {
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
                *trans = init;
                let mut tmp = beziers.get_mut(parent.0).expect("No parent found");
                let off = curve_offset(tmp.ty());
                tmp.update(state.pt, init.translation - off);
            }
        }
    }
}

fn modify_beziers(
    mut modifications: EventReader<BezierModificaiton>,
    mut commands: Commands,
    mut objects: Query<(&mut DragState, &mut Transform, &Parent)>,
    mut beziers: Query<&mut PolyBezier<CubicBezier>>,
    mut sections: Query<(&mut Handle<StandardMaterial>, &Parent), With<BezierSection>>,
    assets: Res<DefaultAssets>,
) {
    for modification in modifications.iter() {
        match modification {
            &BezierModificaiton::Extrude(e, pt) => {
                for (mut state, _t, parent) in objects.iter_mut() {
                    if parent.0 == e && state.pt >= pt {
                        state.pt += 1;
                    }
                }
                let bez = beziers.get(e).unwrap();
                println!(
                    "Extrude: {}, {}, {:?}",
                    bez.get_control_point(pt),
                    pt,
                    bez.ty()
                );
                let child = commands
                    .spawn_bundle(PbrBundle {
                        mesh: assets.handle_mesh.clone(),
                        material: assets.handle_material.clone(),
                        transform: Transform::from_translation(
                            bez.get_control_point(pt) + curve_offset(bez.ty()),
                        ),
                        ..Default::default()
                    })
                    .insert_bundle(bevy_mod_picking::PickableBundle::default())
                    .insert(DragState {
                        pt,
                        ..DragState::default()
                    })
                    .id();
                commands.entity(e).add_child(child);
            }
            &BezierModificaiton::Place(origin, dir) => {
                // TODO: calcuate a better inital starting point and curve type
                let start = origin + dir * 10.;
                let ty = SplineType::TrackBed;

                let mut entity = commands.spawn_bundle(ParentBundle::default());
                entity.with_children(|commands| {
                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: assets.handle_mesh.clone(),
                            material: assets.handle_material.clone(),
                            transform: Transform::from_translation(start + curve_offset(ty)),
                            ..Default::default()
                        })
                        .insert_bundle(bevy_mod_picking::PickableBundle::default())
                        .insert(DragState {
                            pt: 0,
                            ..DragState::default()
                        });
                    let transform = Transform::from_translation(start + curve_offset(ty));
                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: assets.handle_mesh.clone(),
                            material: assets.handle_material.clone(),
                            transform,
                            ..Default::default()
                        })
                        .insert_bundle(bevy_mod_picking::PickableBundle::default())
                        .insert(DragState {
                            pt: 1,
                            drag_start: Some((start, dir)),
                            initial: Some(transform),
                        });
                });
                let bezier = PolyBezier::new(vec![start, start], ty);
                entity.insert(bezier);
            }
            BezierModificaiton::ChangeTy(e, old,  ty) => {
                for (mut mat, parent) in sections.iter_mut() {
                    if &parent.0 == e {
                        *mat = assets.spline_material[*ty].clone();
                    }
                }
                let handle_diff = curve_offset(*ty) - curve_offset(*old);
                if handle_diff != Vec3::ZERO {
                    for (_state, mut trans, parent) in objects.iter_mut() {
                        if &parent.0 == e {
                            trans.translation += handle_diff;
                        }
                    }
                }
            }
            BezierModificaiton::Delete(e, pt) => {
                todo!("delete");
            }
        }
    }
}

fn update_curve_sections(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<DefaultAssets>,
    mut beziers: Query<&mut PolyBezier<CubicBezier>>,
    mut sections: Query<(&mut Transform, &BezierSection)>,
) {
    let start = Instant::now();
    for mut bezier in beziers.iter_mut() {
        for mesh in bezier.create_meshes(&mut meshes, &server) {
            commands
                .spawn_bundle(PbrBundle {
                    mesh: mesh.clone(),
                    material: assets.spline_material[bezier.ty()].clone(),
                    ..Default::default()
                })
                .insert_bundle(bevy_mod_picking::PickableBundle::default())
                .insert(BezierSection(mesh));
        }
        for (translation, mesh) in bezier.get_transforms() {
            for (mut trans, section) in sections.iter_mut() {
                if mesh.has(&section.0) {
                    trans.translation = translation;
                    break;
                }
            }
        }
        // if start.elapsed() > Duration::from_millis(20) {
        //     break;
        // }
    }
}

//         if mouse_opts.action == MouseAction::Extrude {
//             if let Some((s, transform)) = objects
//                 .iter()
//                 .find_map(|(s, _, _)| s.initial.map(|i| (s, i)))
//             {
//                 let id = s.id;
//                 let pt = s.pt;
//                 for (mut s, _, _) in objects.iter_mut() {
//                     if s.id == id && s.pt > pt {
//                         s.pt += 1;
//                     }
//                 }
//                 for mut handle in beziers.iter_mut() {
//                     if handle.0 == id {
//                         handle.1.insert(pt, transform.translation);
//                     }
//                 }
//                 commands
//                     .spawn_bundle(PbrBundle {
//                         mesh: meshes.add(Mesh::from(shape::Cube { size: 3. })),
//                         material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
//                         transform,
//                         ..Default::default()
//                     })
//                     .insert_bundle(bevy_mod_picking::PickableBundle::default())
//                     .insert(DragState {
//                         id,
//                         pt: pt + 1,
//                         ..DragState::default()
//                     });
