use std::cmp::Ordering;

use bevy::prelude::*;

use crate::{
    gvas::{SwitchData, SwitchType},
    spline::{mesh::curve_offset, CubicBezier, PolyBezier},
    update::DragState,
};
// Snap points

pub struct SnapPlugin;

impl Plugin for SnapPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SnapEvent>();
        app.add_system(snap_handler);
    }
}

#[derive(Debug)]
pub enum SnapEvent {
    Spline(Entity, Entity),
    Switch(Entity),
}

fn snap_handler(
    mut splines: Query<&mut PolyBezier<CubicBezier>>,
    mut objects: Query<(&mut Transform, &DragState)>,
    mut switches: Query<(&mut Transform, &SwitchData), Without<DragState>>,
    mut event_reader: EventReader<SnapEvent>,
) {
    for event in event_reader.iter() {
        match event {
            &SnapEvent::Spline(curve, handle) => {
                let off = curve_offset(splines.get(curve).unwrap().ty());
                let (trans, _) = objects.get(handle).unwrap();
                let pt = find_nearest(trans.translation - off, &splines, &switches);
                if pt != trans.translation - off {
                    let (mut handle, state) = objects.get_mut(handle).unwrap();
                    let mut curve = splines.get_mut(curve).unwrap();
                    handle.translation = pt + off;
                    curve.update(state.pt, pt);
                }
            }
            &SnapEvent::Switch(switch) => {
                let (trans, _s) = switches.get(switch).unwrap();
                let pt = find_nearest(trans.translation, &splines, &switches);
                if pt != trans.translation {
                    let (mut handle, _s) = switches.get_mut(switch).unwrap();
                    handle.translation = pt;
                }
            }
        }
    }
}

// const fn vec3_new(x: f32, y: f32, z: f32) -> Vec3 {
//     Vec3::X * x + Vec3::Y * y + Vec3::Z * z
// }


fn find_nearest(
    pt: Vec3,
    splines: &Query<&mut PolyBezier<CubicBezier>>,
    switches: &Query<(&mut Transform, &SwitchData), Without<DragState>>,
) -> Vec3 {
    /// Comparison function to compare by distance
    fn compare(a: &(Vec3, f32), b: &(Vec3, f32)) -> Ordering {
        a.1.partial_cmp(&b.1).unwrap()
    }
    if let Some((v, dist)) = splines
        .iter()
        .flat_map(|s| s.get_control_points())
        .chain(switches.iter().flat_map(|(t, s)| {
            match s.ty {
                SwitchType::Crossover90 => vec![
                    t.translation,
                    t.translation + t.rotation.mul_vec3(Vec3::new(0.38385, 0., 0.)),
                    t.translation
                        + t.rotation
                            .mul_vec3(Vec3::new(0.38385 / 2., 0.38385 / 2., 0.)),
                    t.translation
                        + t.rotation
                            .mul_vec3(Vec3::new(0.38385 / 2., -0.38385 / 2., 0.)),
                ]
                .into_iter(),
                SwitchType::SwitchLeft | SwitchType::SwitchLeftAlt => vec![
                    t.translation,
                    t.translation + t.rotation.mul_vec3(Vec3::new(1.86489, 0., 0.)),
                    t.translation + t.rotation.mul_vec3(Vec3::new(1.86489, 0., 0.)),
                ]
                .into_iter(),
                SwitchType::SwitchRight | SwitchType::SwitchRightAlt => vec![
                    t.translation,
                    t.translation + t.rotation.mul_vec3(Vec3::new(1.86489, 0., 0.)),
                    t.translation + t.rotation.mul_vec3(Vec3::new(1.86489, 0., 0.)),
                ]
                .into_iter(),
            }
        }))
        .filter(|v| v != &pt)
        .map(|v| (v, pt.distance_squared(v)))
        .min_by(compare)
    {
        if dist < 0.2 {
            v
        } else {
            pt
        }
    } else {
        pt
    }
}

// Initial Starting Point: (8.360041, 10.037501, 1.2449101)
// Direction: (8.360041, 10.037501, 1.2449101) -> (10.224866, 10.036594, 1.2599658) = (0.999967, -0.000486357, 0.00807325)

// Length of a switch: ||(8.360041, 10.037501, 1.2449101) -> (10.224866, 10.036594, 1.2599658)|| = 1.86489
// Right -> (10.20949, 10.063137, 1.0753591)
// Left -> (10.369944, 10.065863, 1.2337539) -> (12.241622, 10.030314, 1.2957464)
// Right_dir -> (10.277252, 10.037501, 0.071335696)

// Real measurements:
// flatcar: 7,856 m   aprox 25 ft 9 inch
// boxcar: 8,2282 m aprox 26ft 8 inch
// hadcart: 2,202 m   aprox 7 ft  7 inch
// betsy: 3,912 m   aprox 12 ft 10 inch
// porter: 4,6135 m   aprox 15 ft 2 inch
// eureka: 8,0213 m  aprox 26 ft 4inch
// eurekas tender: 4,9708 m   aprox 15 ft 4 inch
// mogul: 8,3783 m  aprox 27 ft 6 inch
// mogul tender: 6,4173 m aprox 21 ft 1 inch
// class 70: 9,3890 maprox 30 ft 10 inch
// class 70 tender: 6,7881 m aprox 22 ft 3 inch
// cross - length: 3,8385 m aprox  12 ft 7 inch
// climax: 8,4989 m aprox 27 ft 11 inch
// heisler: 9,1373 m aprox 30 ft (0 inch)
// max track length: 10,5 m  aprox 34 ft 5 inch
// straight part of switch: 18,8 m aprox  61 ft 8 inch

// width of flatcar: 1,9327 m aprox 6 ft 4 inch
