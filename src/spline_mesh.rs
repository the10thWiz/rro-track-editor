
use crate::gvas::SplineType;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use crate::curve::CurvePoint;

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

pub fn curve_color(ty: SplineType) -> Color {
    match ty {
        SplineType::GroundWork | SplineType::ConstGroundWork => Color::rgb(0.8, 0.7, 0.6),
        _ => Color::rgb(0.8, 0.7, 0.6),
    }
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
pub fn gen_mesh_from_curve<const N: usize>(
    loc: Vec3,
    curve: impl Iterator<Item = CurvePoint>,
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
