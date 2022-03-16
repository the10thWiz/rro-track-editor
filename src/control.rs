use crate::gvas::{gvas_to_vec, vec_to_gvas, CurveDataOwned, RROSave, SplineType, SwitchData, rotator_to_quat, quat_to_rotator, SwitchType};
use crate::palette::FileEvent;
use crate::spline::mesh::curve_offset;
use crate::spline::{CubicBezier, PolyBezier};
use crate::update::{BezierModificaiton, DragState, UpdatePlugin, BezierSectionUpdate, SwitchDrag};
use bevy::prelude::*;
use enum_map::{enum_map, EnumMap};
use std::fs::File;
use std::path::PathBuf;

/// Plugin for loading, saving, and updates
pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_assets);
        app.insert_resource(
            RROSave::read(&mut std::io::Cursor::new(include_bytes!(
                "../assets/default.sav"
            )))
            .expect("Failed to parse included save"),
        );
        app.add_event::<BezierModificaiton>();
        app.add_system(load_save);
        app.add_plugin(UpdatePlugin);
        // app.add_system(update_bezier_transform);
        // app.add_system(update_curve_sections);
        // app.add_system(modify_beziers);
        // app.add_system(debugging);
    }
}

/// Default Assets, to prevent duplicate assets where possible
pub struct DefaultAssets {
    pub handle_mesh: Handle<Mesh>,
    pub handle_material: Handle<StandardMaterial>,
    pub spline_mesh: EnumMap<SplineType, Handle<Mesh>>,
    pub spline_material: EnumMap<SplineType, Handle<StandardMaterial>>,
    pub hidden_spline_material: EnumMap<SplineType, Handle<StandardMaterial>>,
    pub switch_mesh: EnumMap<SwitchType, Handle<Mesh>>,
    pub switch_material: EnumMap<SwitchType, Handle<StandardMaterial>>,
}

fn init_assets(
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let handle_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.3 }));
    let handle_material = materials.add(Color::rgb(0.8, 0.0, 0.0).into());
    let spline_colors = enum_map! {
            SplineType::GroundWork => Color::rgb(0.8, 0.7, 0.6),
            SplineType::ConstGroundWork => Color::rgb(0.8, 0.7, 0.6),
            SplineType::Track => Color::rgb(0.8, 0.7, 0.6),
            SplineType::TrackBed => Color::rgb(0.8, 0.7, 0.6),
            SplineType::WoodBridge => Color::rgb(0.8, 0.7, 0.6),
            SplineType::SteelBridge => Color::rgb(0.8, 0.7, 0.6),
            SplineType::StoneGroundWork => Color::rgb(0.8, 0.7, 0.6),
            SplineType::ConstStoneGroundWork => Color::rgb(0.8, 0.7, 0.6),
    };
    let spline_mesh = enum_map! {
        SplineType::Track => asset_server.load("models/track.obj"),
        SplineType::TrackBed => asset_server.load("models/tube.obj"),
        SplineType::WoodBridge => asset_server.load("models/tube.obj"),
        SplineType::SteelBridge => asset_server.load("models/tube.obj"),
        SplineType::GroundWork | SplineType::ConstGroundWork => asset_server.load("models/groundwork.obj"),
        SplineType::StoneGroundWork | SplineType::ConstStoneGroundWork => asset_server.load("models/stonewall.obj"),
    };
    let spline_material = spline_colors.map(|_k, e| materials.add(e.into()));
    let hidden_spline_material = spline_colors.map(|_k, mut e| {
        e.set_a(0.3);
        materials.add(e.into())
    });
    let switch_mesh = enum_map! {
        SwitchType::Crossover90 => asset_server.load("models/tube.obj"),
        _ => asset_server.load("models/switch.obj"),
    };
    let switch_material = enum_map! {
        _ => materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    };
    commands.insert_resource(DefaultAssets {
        handle_mesh,
        handle_material,
        spline_mesh,
        spline_material,
        hidden_spline_material,
        switch_mesh,
        switch_material,
    });
}

fn load_save(
    mut events: EventReader<FileEvent>,
    assets: Res<DefaultAssets>,
    beziers: Query<(Entity, &PolyBezier<CubicBezier>, &Children)>,
    switches: Query<(Entity, &Transform, &SwitchData)>,
    mut gvas: ResMut<RROSave>,
    mut commands: Commands,
    mut section_update: EventWriter<BezierSectionUpdate>,
) {
    for event in events.iter() {
        if let Err(e) = match event {
            FileEvent::Load(path) => {
                load_file(path, &assets, &beziers, &switches, &mut commands, &mut section_update)
            }
            FileEvent::Save(path) => save_file(path, &beziers, &switches, &mut gvas),
        } {
            println!("Error: {:?}", e);
        }
    }
}

/// The minimal set of components to create an empty parent for meshes
#[derive(Debug, Bundle, Default)]
pub struct ParentBundle {
    _local: Transform,
    _global: GlobalTransform,
}

fn save_file(
    path: &PathBuf,
    beziers: &Query<(Entity, &PolyBezier<CubicBezier>, &Children)>,
    switches: &Query<(Entity, &Transform, &SwitchData)>,
    gvas: &mut ResMut<RROSave>,
) -> Result<(), crate::gvas::GVASError> {
    gvas.set_curves(beziers.iter().map(|(_e, b, _c)| {
        let control_points: Vec<_> = b.get_control_points().map(|v| vec_to_gvas(v)).collect();
        CurveDataOwned {
            location: control_points[0],
            ty: b.ty(),
            visibility: vec![true; control_points.len() - 1],
            control_points,
        }
    }))?;
    gvas.set_switches(switches.iter().map(|(_e, t, s)| {
        let mut tmp = *s;
        tmp.location = vec_to_gvas(t.translation);
        tmp.rotation = quat_to_rotator(t.rotation);
        tmp
    }))?;
    gvas.write(&mut File::create(path)?)?;
    Ok(())
}

fn load_file(
    path: &PathBuf,
    assets: &Res<DefaultAssets>,
    beziers: &Query<(Entity, &PolyBezier<CubicBezier>, &Children)>,
    switches: &Query<(Entity, &Transform, &SwitchData)>,
    commands: &mut Commands,
    section_update: &mut EventWriter<BezierSectionUpdate>,
) -> Result<(), crate::gvas::GVASError> {
    // Clear the world
    for (e, _c, children) in beziers.iter() {
        commands.entity(e).despawn();
        for child in children.iter() {
            commands.entity(*child).despawn();
        }
    }
    for (e, _t, _s) in switches.iter() {
        commands.entity(e).despawn();
    }
    // Load from file
    let gvas = crate::gvas::RROSave::read(&mut File::open(path)?)?;
    for curve in gvas.curves()? {
        // TODO: spawn curves
        let mut entity = commands.spawn_bundle(ParentBundle::default());
        let points: Vec<_> = curve
            .control_points
            .iter()
            .map(|arr| gvas_to_vec(*arr))
            .collect();
        entity.with_children(|commands| {
            for (i, point) in points.iter().enumerate() {
                println!("Spawning point at {}", point);
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: assets.handle_mesh.clone(),
                        material: assets.handle_material.clone(),
                        transform: Transform::from_translation(*point + curve_offset(curve.ty)),
                        ..Default::default()
                    })
                    .insert_bundle(bevy_mod_picking::PickableBundle::default())
                    .insert(DragState::new(i));
            }
        });
        let bezier = PolyBezier::new(points, curve.visibility.iter().copied().collect(), curve.ty);
        entity.insert(bezier);
        section_update.send(BezierSectionUpdate { bezier: entity.id() });
    }
    for switch in gvas.switches()? {
        commands
            .spawn_bundle(PbrBundle {
                mesh: assets.switch_mesh[switch.ty].clone(),
                material: assets.switch_material[switch.ty].clone(),
                transform: Transform {
                    translation: gvas_to_vec(switch.location),
                    scale: switch.ty.scale(),
                    rotation: rotator_to_quat(switch.rotation),
                },
                ..Default::default()
            })
            .insert_bundle(bevy_mod_picking::PickableBundle::default())
            .insert(SwitchDrag::default())
            .insert(switch);
    }
    commands.insert_resource(gvas);
    Ok(())
}