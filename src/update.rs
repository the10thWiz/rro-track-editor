use crate::control::{DefaultAssets, ParentBundle};
use crate::gvas::{SplineType, SwitchData};
use crate::palette::{DebugInfo, MouseAction, Palette};
use crate::spline::mesh::curve_offset;
use crate::spline::{CubicBezier, PolyBezier};
use bevy::prelude::*;
use bevy_mod_picking::{Hover, PickingCamera};
use std::time::{Duration, Instant};

pub struct UpdatePlugin;

impl Plugin for UpdatePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BezierSectionUpdate>();
        app.add_system(update_bezier_transform);
        app.add_system(update_curve_sections);
        app.add_system(modify_beziers);
        app.add_system(debugging);
    }
}

#[derive(Debug, Component, Default)]
pub struct DragState {
    pt: usize,
    drag_start: Option<(Vec3, Vec3, Vec3)>,
    initial: Option<Transform>,
}

impl DragState {
    pub fn new(pt: usize) -> Self {
        Self {
            pt,
            ..Default::default()
        }
    }
}

#[derive(Debug, Component, Default)]
pub struct BezierSection(Handle<Mesh>);

#[derive(Debug, Clone, PartialEq)]
pub enum BezierModificaiton {
    Extrude(Entity, usize),
    DeletePt(Entity, usize),
    DeleteSection(Entity, Handle<Mesh>),
    Place(Vec3, Vec3),
    ChangeTy(Entity, SplineType, SplineType),
    ChangeVis(Entity, SplineType, bool),
}

fn debugging(
    state: Res<Palette>,
    objects: Query<(&Hover, &Transform, &Parent, &DragState)>,
    sections: Query<(&Hover, &Parent, &BezierSection)>,
    beziers: Query<&PolyBezier<CubicBezier>>,
    switches: Query<(&Hover, &Transform, &SwitchData)>,
    mut debug_info: ResMut<DebugInfo>,
) {
    if state.show_debug {
        let mut has_hover = false;
        for (hover, trans, parent, state) in objects.iter() {
            if hover.hovered() {
                let bez = beziers.get(parent.0.clone()).unwrap();
                has_hover = true;
                debug_info.hovered = format!(
                    "Point: {}\nty: {:?}\npt: {}",
                    trans.translation - curve_offset(bez.ty()),
                    bez.ty(),
                    state.pt
                );
            }
        }
        for (hover, trans, state) in switches.iter() {
            if hover.hovered() {
                has_hover = true;
                debug_info.hovered = format!("Switch: {:?}\ntrans: {:?}", state, trans);
            }
        }
        for (hover, parent, section) in sections.iter() {
            if hover.hovered() {
                let bez = beziers.get(parent.0.clone()).unwrap();
                has_hover = true;
                debug_info.hovered = format!(
                    "Bezier: {:?}\nsegement: {:?}\nmodified: {:?}",
                    bez.get_control_points().collect::<Vec<_>>(),
                    bez.get_segment(&section.0),
                    bez.get_modified()
                );
            }
        }
        if !has_hover && debug_info.hovered != "" {
            debug_info.hovered = format!("");
        }
    }
}

fn update_bezier_transform(
    pick_cam: Query<&PickingCamera>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut objects: Query<(&mut DragState, &Hover, &mut Transform, &Parent)>,
    sections: Query<(&Hover, &Parent, &BezierSection, Entity)>,
    mut beziers: Query<&mut PolyBezier<CubicBezier>>,
    mut palette: ResMut<Palette>,
    mut modification: EventWriter<BezierModificaiton>,
    mut section_update: EventWriter<BezierSectionUpdate>,
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
            for (mut state, hover, trans, _p) in objects.iter_mut() {
                if hover.hovered() {
                    state.initial = Some(trans.clone());
                    let dir = if palette.lock_z {
                        Vec3::new(0., 1., 0.)
                    } else {
                        picking_ray.direction()
                    };
                    let tmp =
                        picking_camera.intersect_primitive(bevy_mod_picking::Primitive3d::Plane {
                            point: trans.translation,
                            normal: dir,
                        });
                    state.drag_start = Some((
                        trans.translation,
                        picking_ray.direction(),
                        tmp.map_or(Vec3::ZERO, |int| int.position() - trans.translation),
                    ));
                }
            }
        } else if matches!(palette.action, MouseAction::Place) {
            modification.send(BezierModificaiton::Place(
                picking_ray.origin(),
                picking_ray.direction(),
            ));
        } else if matches!(palette.action, MouseAction::Delete) {
            let mut found_hover = false;
            for (state, hover, _trans, parent) in objects.iter() {
                if hover.hovered() {
                    modification.send(BezierModificaiton::DeletePt(parent.0.clone(), state.pt));
                    found_hover = true;
                    break;
                }
            }
            if !found_hover {
                for (hover, parent, sec, _e) in sections.iter() {
                    if hover.hovered() {
                        modification.send(BezierModificaiton::DeleteSection(
                            parent.0.clone(),
                            sec.0.clone(),
                        ));
                        break;
                    }
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
        } else if matches!(palette.action, MouseAction::ToggleVisibility) {
            for (hover, parent, section, entity) in sections.iter() {
                if hover.hovered() {
                    let mut bez = beziers.get_mut(parent.0.clone()).unwrap();
                    let vis = bez.toggle_segment_visible(&section.0);
                    modification.send(BezierModificaiton::ChangeVis(entity, bez.ty(), vis));
                }
            }
        }
    } else if mouse_button_input.just_released(MouseButton::Left) {
        for (mut state, _sel, _trans, parent) in objects.iter_mut() {
            state.initial = None;
            state.drag_start = None;
            section_update.send(BezierSectionUpdate {
                bezier: parent.0.clone(),
            });
        }
        // Clicking on a piece of track forces an update
        for (hover, parent, _, _) in sections.iter() {
            if hover.hovered() {
                section_update.send(BezierSectionUpdate {
                    bezier: parent.0.clone(),
                });
            }
        }
    }

    for (state, _sel, mut trans, parent) in objects.iter_mut() {
        if let Some((origin, dir, offset)) = state.drag_start {
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
                let dir = int.position() - origin - offset;
                let mut init = match state.initial {
                    Some(initial) => initial,
                    None => unreachable!(),
                };
                init.translation += dir;
                *trans = init;
                let mut bez = beziers.get_mut(parent.0).expect("No parent found");
                let off = curve_offset(bez.ty());
                if dir != Vec3::ZERO {
                    if matches!(palette.action, MouseAction::Extrude) {
                        let loc = init.translation - off;
                        let before = bez.before(state.pt, init.translation);
                        println!(
                            "Before: {}, pt: {} -> {}",
                            before,
                            state.pt,
                            state.pt + if !before { 1 } else { 0 }
                        );
                        bez.insert(state.pt + if !before { 1 } else { 0 }, loc);
                        modification.send(BezierModificaiton::Extrude(parent.0.clone(), state.pt));
                        palette.action = MouseAction::Drag;
                    }
                }
                bez.update(state.pt, init.translation - off);
                // println!("Sending update");
                section_update.send(BezierSectionUpdate {
                    bezier: parent.0.clone(),
                });
            }
        }
    }
}

fn modify_beziers(
    mut modifications: EventReader<BezierModificaiton>,
    mut commands: Commands,
    mut objects: Query<(&mut DragState, &mut Transform, &Parent, Entity)>,
    beziers: Query<(&PolyBezier<CubicBezier>, Entity, &Children)>,
    mut sections: Query<(
        &mut Handle<StandardMaterial>,
        Entity,
        &Parent,
        &BezierSection,
    )>,
    assets: Res<DefaultAssets>,
    mut section_update: EventWriter<BezierSectionUpdate>,
) {
    for modification in modifications.iter() {
        match modification {
            &BezierModificaiton::Extrude(e, pt) => {
                for (mut state, _t, parent, _e) in objects.iter_mut() {
                    if parent.0 == e && state.pt >= pt {
                        state.pt += 1;
                    }
                }
                let (bez, _e, _c) = beziers.get(e).unwrap();
                let loc = bez.get_control_point(pt);
                println!("Extrude: {}, {}, {:?}", loc, pt, bez.ty());
                // bez.insert(pt, loc);
                let child = commands
                    .spawn_bundle(PbrBundle {
                        mesh: assets.handle_mesh.clone(),
                        material: assets.handle_material.clone(),
                        transform: Transform::from_translation(loc + curve_offset(bez.ty())),
                        ..Default::default()
                    })
                    .insert_bundle(bevy_mod_picking::PickableBundle::default())
                    .insert(DragState {
                        pt,
                        ..DragState::default()
                    })
                    .id();
                commands.entity(e).add_child(child);
                section_update.send(BezierSectionUpdate { bezier: e });
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
                            drag_start: Some((start, dir, Vec3::ZERO)),
                            initial: Some(transform),
                        });
                });
                let bezier = PolyBezier::new(vec![start, start], vec![true, true], ty);
                entity.insert(bezier);
                section_update.send(BezierSectionUpdate {
                    bezier: entity.id(),
                });
            }
            BezierModificaiton::ChangeTy(e, old, ty) => {
                for (mut mat, _e, parent, s) in sections.iter_mut() {
                    if &parent.0 == e {
                        let (bez, _, _) = beziers.get(parent.0.clone()).unwrap();
                        if bez.segment_visible(&s.0) {
                            *mat = assets.spline_material[*ty].clone();
                        } else {
                            *mat = assets.hidden_spline_material[*ty].clone();
                        }
                    }
                }
                let handle_diff = curve_offset(*ty) - curve_offset(*old);
                if handle_diff != Vec3::ZERO {
                    for (_state, mut trans, parent, _e) in objects.iter_mut() {
                        if &parent.0 == e {
                            trans.translation += handle_diff;
                        }
                    }
                }
            }
            BezierModificaiton::ChangeVis(e, ty, vis) => {
                let (mut mat, _e, _p, _s) = sections.get_mut(e.clone()).unwrap();
                if *vis {
                    *mat = assets.spline_material[*ty].clone();
                } else {
                    *mat = assets.hidden_spline_material[*ty].clone();
                }
            }
            BezierModificaiton::DeletePt(e, pt) => {
                let (first, entity, children) = beziers.get(e.clone()).unwrap();
                let (first, second) = first.split_pt(*pt);
                commands.entity(entity).despawn();
                for child in children.iter() {
                    commands.entity(child.clone()).despawn();
                }
                if let Some(bezier) = spawn_bezier(&mut commands, &assets, first) {
                    section_update.send(BezierSectionUpdate { bezier });
                }
                if let Some(bezier) = spawn_bezier(&mut commands, &assets, second) {
                    section_update.send(BezierSectionUpdate { bezier });
                }
            }
            BezierModificaiton::DeleteSection(e, section) => {
                let (first, entity, children) = beziers.get(e.clone()).unwrap();
                let (first, second) = first.split_sec(section);
                commands.entity(entity).despawn();
                for child in children.iter() {
                    commands.entity(child.clone()).despawn();
                }
                if let Some(bezier) = spawn_bezier(&mut commands, &assets, first) {
                    section_update.send(BezierSectionUpdate { bezier });
                }
                if let Some(bezier) = spawn_bezier(&mut commands, &assets, second) {
                    section_update.send(BezierSectionUpdate { bezier });
                }
            }
        }
    }
}

fn spawn_bezier(
    commands: &mut Commands,
    assets: &DefaultAssets,
    first: PolyBezier<CubicBezier>,
) -> Option<Entity> {
    if first.len() > 1 {
        let mut entity = commands.spawn_bundle(ParentBundle::default());
        entity.with_children(|commands| {
            for (pt, loc) in first.get_control_points().enumerate() {
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: assets.handle_mesh.clone(),
                        material: assets.handle_material.clone(),
                        transform: Transform::from_translation(loc + curve_offset(first.ty())),
                        ..Default::default()
                    })
                    .insert_bundle(bevy_mod_picking::PickableBundle::default())
                    .insert(DragState {
                        pt,
                        ..DragState::default()
                    });
            }
        });
        entity.insert(first);
        Some(entity.id())
    } else {
        None
    }
}

pub struct BezierSectionUpdate {
    pub bezier: Entity,
}

fn update_curve_sections(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<DefaultAssets>,
    mut beziers: Query<&mut PolyBezier<CubicBezier>>,
    mut sections: Query<(&mut Transform, &BezierSection)>,
    mut section_update: EventReader<BezierSectionUpdate>,
) {
    let start = Instant::now();
    for update in section_update.iter() {
        let entity = update.bezier.clone();
        if let Ok(mut bezier) = beziers.get_mut(entity) {
            // println!("Has update: {:?}", bezier.ty());
            // println!("Bez: {:?}", bezier);
            for (mesh, visible) in bezier.create_meshes(&mut meshes, &server) {
                let section = commands
                    .spawn_bundle(PbrBundle {
                        mesh: mesh.clone(),
                        material: if visible {
                            assets.spline_material[bezier.ty()].clone()
                        } else {
                            assets.hidden_spline_material[bezier.ty()].clone()
                        },
                        ..Default::default()
                    })
                    .insert_bundle(bevy_mod_picking::PickableBundle::default())
                    .insert(BezierSection(mesh))
                    .id();
                commands.entity(entity).add_child(section);
            }
            for (translation, mesh) in bezier.get_transforms() {
                for (mut trans, section) in sections.iter_mut() {
                    if mesh.has(&section.0) {
                        trans.translation = translation;
                        break;
                    }
                }
            }
            if start.elapsed() > Duration::from_millis(20) {
                // TODO: avoid this and enable partial application?
                // I don't actually overrun that often, but Bevy doesn't really update as fast as I'd like here
                // This should actually be handled by some kind of event system, so I only loop through the ones
                // that need to be updates.
                println!("Task overrun");
                break;
            }
        }
    }
}
