
use crate::gvas::SplineType;
use bevy::{prelude::*, render::mesh::VertexAttributeValues};
use bevy::render::mesh::Indices;
use bevy::math::Vec4Swizzles;
use enum_map::{EnumMap, enum_map};
use crate::spline::CurvePoint;

use super::{Bezier, CubicBezier};

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

pub fn spline_mesh(ty: SplineType) -> &'static str {
    match ty {
        SplineType::Track => "models/track.obj",
        SplineType::TrackBed => "models/tube.obj",
        SplineType::WoodBridge => "models/tube.obj",
        SplineType::SteelBridge => "models/tube.obj",
        SplineType::GroundWork | SplineType::ConstGroundWork => "models/groundwork.obj",
        SplineType::StoneGroundWork | SplineType::ConstStoneGroundWork => "models/stonewall.obj",
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

/// inverse bisect between ba and bc
fn bisect_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let ba = a - b;
    let bc = c - b;
    let bisect = ba * bc.length() + bc * ba.length();
    -bisect.normalize()
}

// Offsets is a 2D shape to use as a cross section of the generated mesh. `y` is vertical according
// to the curve.
pub fn gen_mesh_from_curve<C: Iterator<Item = CurvePoint>, const N: usize>(
    loc: Vec3,
    curve: C,
    offsets: [Vec2; N],
) -> Mesh {
    let mut points = vec![];
    let mut normals = vec![];
    let mut uv = vec![];
    for mut pt in curve {
        pt.point -= loc;
        const SIZE: f32 = 1.;
        pt.normal = pt.normal.normalize() * SIZE;
        pt.tangent = pt.tangent.normalize() * SIZE;
        pt.up = pt.up.normalize() * SIZE;
        // This was trying to make sure the points always had the same winding order, but they seem
        // to anyway
        //println!("{}", pt.tangent.cross(pt.normal));
        //println!("{}", pt.normal);
        if pt.tangent.cross(pt.normal).y > 0. {
            //panic!("{}", pt.tangent.cross(pt.normal));
        }
        let mut pts = [Vec3::ZERO; N];
        for (i, offset) in offsets.iter().enumerate() {
            pts[i] = pt.point + pt.normal * offset.x + pt.up * offset.y;
        }

        // Debug assert that we were not given the same point twice
        debug_assert_ne!(pts[N - 1].to_array(), *points.last().unwrap_or(&[0.; 3]));
        points.extend(pts.into_iter().map(|p| p.to_array()));

        normals.push(bisect_normal(pts[N - 1], pts[0], pts[1]).to_array());
        for i in 2..N {
            normals.push(bisect_normal(pts[i - 2], pts[i - 1], pts[i]).to_array());
        }
        normals.push(bisect_normal(pts[N - 2], pts[N - 1], pts[0]).to_array());

        for i in 0..N {
            uv.push([i as f32 / (N - 1) as f32, pt.t]);
        }
        //print!(".");
    }
    //println!();
    let num_pts = (points.len() - N) as u32;
    let mut indicies = vec![
        //0, 2, 1,
        //0, 3, 2,
        //num_pts + 0, num_pts + 1, num_pts + 2,
        //num_pts + 0, num_pts + 2, num_pts + 3,
        // TODO: consider closing ends
    ];
    for i in (0..num_pts).step_by(N) {
        for n in 0..N as u32 {
            let pt = i + n;
            let pt_next = i + (n + 1) % N as u32;
            indicies.extend([pt_next, pt + 0, pt + N as u32]);
            indicies.extend([pt_next, pt + N as u32, pt_next + N as u32]);
        }
    }
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, points);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uv);
    mesh.set_indices(Some(Indices::U32(indicies)));
    mesh
}

pub fn mesh_from_curve(loc: Vec3, curve: impl Iterator<Item = CurvePoint>, ty: SplineType) -> Mesh {
    match ty {
        SplineType::GroundWork | SplineType::ConstGroundWork => gen_mesh_from_curve(
            loc,
            curve,
            [
                Vec2::new(-0.3, -0.2),
                Vec2::new(0.1, -0.2),
                Vec2::new(0., 0.),
                Vec2::new(-0.2, 0.),
            ],
        ),
        _ => gen_mesh_from_curve(
            loc,
            curve,
            [
                Vec2::new(-0.1, -0.1),
                Vec2::new(0.1, -0.1),
                Vec2::new(0.1, 0.1),
                Vec2::new(-0.1, 0.1),
            ],
        ),
    }
    
}
