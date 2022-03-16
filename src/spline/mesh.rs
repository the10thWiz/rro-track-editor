
use crate::gvas::SplineType;
use bevy::{prelude::*, render::mesh::VertexAttributeValues};
use bevy::math::Vec4Swizzles;

use super::CubicBezier;

pub fn curve_offset(ty: SplineType) -> Vec3 {
    match ty {
        SplineType::Track => Vec3::new(0., 1., 0.),
        SplineType::TrackBed => Vec3::new(0., 1., 0.),
        SplineType::WoodBridge => Vec3::new(0., 0., 0.),
        SplineType::SteelBridge => Vec3::new(0., 0., 0.),
        SplineType::GroundWork => Vec3::new(0., 0., 0.),
        SplineType::ConstGroundWork => Vec3::new(0., 0., 0.),
        SplineType::StoneGroundWork => Vec3::new(0., 0., 0.),
        SplineType::ConstStoneGroundWork => Vec3::new(0., 0., 0.),
    }
}

fn matrix_between(a: Vec3, b: Vec3) -> Mat4 {
    let x = b - a;
    let y = Vec3::new(0., 1., 0.);
    let z = x.cross(y).normalize();
    Mat4::from_cols(Vec4::from((x, 0.)), Vec4::from((y, 0.)), Vec4::from((z, 0.)), Vec4::from((a, 1.)))
}

fn bend_mesh_on_curve(loc: Vec3, curve: &CubicBezier, points: &mut Vec<[f32; 3]>, normals: &mut Vec<[f32; 3]>) {
    // Step one: Express points and normals as a function of a bezier curve. Then undo, but with the provided curve.
    // Trivialize by aligning the initial points such that one coordinate represents the distance along the curve.
    // const LENGTH: f32 = 10.;
    // let up = Vec3::new(0., 1., 0.);
    // for (point, normal) in points.iter_mut().zip(normals.iter_mut()) {
    //     let dist = point[0] / LENGTH;
    //     let height = point[1];
    //     let right = point[2];
    //     let pt = curve.eval(dist);
    //     let pt = pt + height * up;
    //     let pt = pt + height * curve.derivative().eval(dist).cross(up);
    //     *point = [pt.x, pt.y, pt.z];
    // }
    let &[a, b, c, d] = curve.get_pts();
    let ab = matrix_between(a, b);
    let bc = matrix_between(b, c);
    let cd = matrix_between(c, d);
    const SCALE_FACTOR: f32 = 10.;
    for (p, n) in points.iter_mut().zip(normals.iter_mut()) {
        let point = Vec4::new(p[0] / SCALE_FACTOR, p[1] / SCALE_FACTOR, p[2] / SCALE_FACTOR, 1.);
        let normal = Vec4::new(n[0], n[1], n[2], 0.);
        let p_ab = ab * point;
        let p_bc = bc * point;
        let p_cd = cd * point;
        let p_abc = matrix_between(p_ab.xyz(), p_bc.xyz()) * point;
        let p_bcd = matrix_between(p_bc.xyz(), p_cd.xyz()) * point;
        let p_mat = matrix_between(p_abc.xyz(), p_bcd.xyz());
        let p_fin = (p_mat * point).xyz() - loc;
        let n_fin = p_mat * normal;
        *p = [p_fin.x, p_fin.y, p_fin.z];
        *n = [n_fin.x, n_fin.y, n_fin.z];
    }
}

pub fn mesh_on_curve(original: &Mesh, loc: Vec3, curve: &CubicBezier) -> Mesh {
    let mut new = original.clone();
    // Safety: This extra mutable reference is used to extract a second attribute.
    // They are guarnteed to be different, since I'm passing different values to `attribute_mut`
    let extra_ref = unsafe { &mut *((&mut new) as *mut Mesh) };
    let points = if let Some(VertexAttributeValues::Float32x3(vec)) = new.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        vec
    } else {
        panic!("Mesh did not have position attribue");
    };
    let normals = if let Some(VertexAttributeValues::Float32x3(vec)) = extra_ref.attribute_mut(Mesh::ATTRIBUTE_NORMAL) {
        vec
    } else {
        panic!("Mesh did not have position attribue");
    };
    bend_mesh_on_curve(loc, curve, points, normals);
    new
}
