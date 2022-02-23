//
// curve.rs
// Copyright (C) 2022 matthew <matthew@matthew-ubuntu>
// Distributed under terms of the MIT license.
//

use std::borrow::Cow;

use bevy::{ecs::system::EntityCommands, prelude::*, render::mesh::Indices};
use bevy_transform_gizmo::TransformGizmoEvent;
use bspline::BSpline;

use crate::BezierSection;

pub struct CurvePoint {
    //points: [Vec3; 4],
    point: Vec3,
    up: Vec3,
    normal: Vec3,
    tangent: Vec3,
    t: f32,
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

pub fn mesh_from_curve(loc: Vec3, curve: impl Iterator<Item = CurvePoint>) -> Mesh {
    gen_mesh_from_curve(
        loc,
        curve,
        [
            Vec2::new(-0.1, -0.1),
            Vec2::new(0.1, -0.1),
            Vec2::new(0.1, 0.1),
            Vec2::new(-0.1, 0.1),
        ],
    )
}

pub trait Bezier: Clone {
    type Derivative: Bezier;

    /// Evaluate the curve at point t
    fn eval(&self, t: f32) -> Vec3;

    fn centroid(&self) -> Vec3;

    fn derivative(&self) -> Self::Derivative;

    fn walker<'a>(&'a self, step: f32, err: f32) -> BezierWalker<'a, Self> {
        BezierWalker {
            curve: self,
            derivative: Cow::Owned(self.derivative()),
            t: 0.,
            step_sq: step * step,
            err_sq: err * err,
            end: 1.,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CubicBezier {
    pts: [Vec3; 4],
}

impl CubicBezier {
    pub fn new(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> Self {
        Self { pts: [a, b, c, d] }
    }

    pub fn update(&mut self, pt: usize, loc: Vec3) {
        self.pts[pt] = loc;
    }

    pub fn transform(&mut self, event: &TransformGizmoEvent) -> bool {
        for p in self.pts.iter_mut() {
            if *p == event.from.translation {
                *p = event.to.translation;
                return true;
            }
        }
        false
    }
}

impl Bezier for CubicBezier {
    type Derivative = QuadraticBezier;
    /// Evaluate the curve at point t
    fn eval(&self, t: f32) -> Vec3 {
        let a = self.pts[0] + (self.pts[1] - self.pts[0]) * t;
        let b = self.pts[1] + (self.pts[2] - self.pts[1]) * t;
        let c = self.pts[2] + (self.pts[3] - self.pts[2]) * t;
        let ab = a + (b - a) * t;
        let bc = b + (c - b) * t;
        ab + (bc - ab) * t
    }

    fn centroid(&self) -> Vec3 {
        (self.pts[0] + self.pts[1] + self.pts[2] + self.pts[3]) / self.pts.len() as f32
    }

    fn derivative(&self) -> QuadraticBezier {
        QuadraticBezier {
            pts: [
                3. * (self.pts[1] - self.pts[0]),
                3. * (self.pts[2] - self.pts[1]),
                3. * (self.pts[3] - self.pts[2]),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct BezierWalker<'a, B: Bezier + Clone + ?Sized> {
    curve: &'a B,
    derivative: Cow<'a, B::Derivative>,
    t: f32,
    step_sq: f32,
    err_sq: f32,
    end: f32,
}

impl<'a, B: Bezier + Clone + ?Sized> Iterator for BezierWalker<'a, B> {
    type Item = CurvePoint;
    fn next(&mut self) -> Option<Self::Item> {
        if self.t >= self.end {
            None
        } else {
            let cur = self.curve.eval(self.t);
            let mut min = self.t;
            let mut max = self.end;
            let (point, guess) = loop {
                let guess = (min + max) / 2.;
                let pt = self.curve.eval(guess);
                let dist = (cur - pt).length_squared() - self.step_sq;
                if dist < -self.err_sq {
                    min = guess;
                } else if dist > self.err_sq {
                    max = guess;
                } else {
                    break (pt, guess);
                }
                if min > self.end - 0.02 {
                    break (self.curve.eval(self.end), self.end);
                }
            };
            self.t = guess;
            let tangent = self.derivative.eval(guess);
            let up = Vec3::new(0.0, 0.1, 0.0);
            let normal = tangent.cross(up).normalize() * 0.1;
            Some(CurvePoint {
                //points: [pt, pt + up, pt + up + normal, pt + normal],
                point,
                up,
                normal,
                tangent,
                t: guess,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuadraticBezier {
    pts: [Vec3; 3],
}

impl QuadraticBezier {
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        Self { pts: [a, b, c] }
    }
}

impl Bezier for QuadraticBezier {
    type Derivative = Line;

    /// Evaluate the curve at point t
    fn eval(&self, t: f32) -> Vec3 {
        let a = self.pts[0] + (self.pts[1] - self.pts[0]) * t;
        let b = self.pts[1] + (self.pts[2] - self.pts[1]) * t;
        a + (b - a) * t
    }

    fn centroid(&self) -> Vec3 {
        (self.pts[0] + self.pts[1] + self.pts[2]) / self.pts.len() as f32
    }

    fn derivative(&self) -> Line {
        Line {
            pts: [
                2. * (self.pts[1] - self.pts[0]),
                2. * (self.pts[2] - self.pts[1]),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    pts: [Vec3; 2],
}

impl Line {
    pub fn new(a: Vec3, b: Vec3) -> Self {
        Self { pts: [a, b] }
    }
}

impl Bezier for Line {
    type Derivative = Vec3;

    /// Evaluate the curve at point t
    fn eval(&self, t: f32) -> Vec3 {
        self.pts[0] + (self.pts[1] - self.pts[0]) * t
    }

    fn centroid(&self) -> Vec3 {
        (self.pts[0] + self.pts[1]) / self.pts.len() as f32
    }

    fn derivative(&self) -> Vec3 {
        self.pts[1] - self.pts[0]
    }
}

impl Bezier for Vec3 {
    type Derivative = Vec3;

    /// Evaluate the curve at point t
    fn eval(&self, t: f32) -> Vec3 {
        *self
    }

    fn centroid(&self) -> Vec3 {
        *self
    }

    fn derivative(&self) -> Vec3 {
        Vec3::new(0., 0., 0.)
    }
}

#[derive(Debug, Clone)]
enum MeshUpdate {
    Insert,
    Modified(Handle<Mesh>),
    None(Handle<Mesh>),
}

impl MeshUpdate {
    pub fn modified(&mut self) {
        match self {
            Self::None(m) => *self = Self::Modified(m.clone()),
            _ => (),
        }
    }

    pub fn is_modified(&self) -> bool {
        match self {
            Self::None(_) => false,
            _ => true,
        }
    }

    pub fn set(
        &mut self,
        assets: &mut Assets<Mesh>,
        f: impl FnOnce() -> Mesh,
    ) -> Option<Handle<Mesh>> {
        match self {
            Self::Insert => {
                let mesh = assets.add(f());
                *self = Self::None(mesh.clone_weak());
                Some(mesh)
            }
            Self::Modified(old) => {
                let mesh = assets.set(old.clone(), f());
                *self = Self::None(mesh.clone_weak());
                None
            }
            Self::None(_) => None,
        }
    }

    pub fn has(&self, h: &Handle<Mesh>) -> bool {
        match self {
            Self::Insert => false,
            Self::None(m) | Self::Modified(m) => m.id == h.id,
        }
    }
}

#[derive(Debug)]
pub struct PolyBezier<C: Bezier> {
    parts: Vec<C>,
    derivative: Option<Vec<C::Derivative>>,
    updates: Vec<MeshUpdate>,
    //meshes: Vec<Handle<Mesh>>,
}

impl<C: Bezier> Clone for PolyBezier<C> {
    fn clone(&self) -> Self {
        Self {
            parts: self.parts.clone(),
            derivative: self.derivative.clone(),
            updates: vec![MeshUpdate::Insert; self.updates.len()],
        }
    }
}

impl PolyBezier<CubicBezier> {
    pub fn new(points: Vec<Vec3>) -> Self {
        assert!(points.len() > 1);
        if points.len() == 2 {
            Self {
                parts: vec![CubicBezier::new(points[0], points[0], points[1], points[1])],
                derivative: None,
                updates: vec![MeshUpdate::Insert],
            }
        } else {
            let mut parts = Vec::with_capacity(points.len() - 1);
            for i in 0..points.len() - 1 {
                parts.push(CubicBezier::new(
                    points[i],
                    Vec3::ZERO,
                    Vec3::ZERO,
                    points[i + 1],
                ));
            }
            let mut ret = Self {
                updates: vec![MeshUpdate::Insert; points.len() - 1],
                derivative: None,
                parts,
            };
            ret.compute_tweens();
            //for (i, p) in points.iter().enumerate() {
            //ret.update(i, *p);
            //}
            //println!("{:?}", ret);
            ret
        }
    }

    pub fn update(&mut self, pt: usize, loc: Vec3) {
        assert!(pt <= self.parts.len());
        if pt == 0 {
            self.parts[0].pts[0] = loc;
            self.updates[0].modified();
            if self.updates.len() > 1 {
                self.updates[1].modified();
            }
        } else if pt == self.parts.len() {
            self.parts[pt - 1].pts[3] = loc;
            self.updates[pt - 1].modified();
            if self.updates.len() > 1 {
                self.updates[pt - 2].modified();
            }
        } else {
            self.parts[pt - 1].pts[3] = loc;
            self.parts[pt].pts[0] = loc;
            if pt > 2 {
                self.updates[pt - 2].modified();
            }
            self.updates[pt - 1].modified();
            self.updates[pt].modified();
            if pt + 1 < self.parts.len() {
                self.updates[pt + 1].modified();
            }
        }
        self.compute_tweens();
    }

    fn compute_tweens(&mut self) {
        for pt in 1..self.parts.len() {
            let tan = (self.parts[pt - 1].pts[0] - self.parts[pt].pts[3]).normalize();
            self.parts[pt - 1].pts[2] = self.parts[pt - 1].pts[3]
                + tan * ((self.parts[pt - 1].pts[0] - self.parts[pt - 1].pts[3]).length() * 0.3);
            self.parts[pt].pts[1] = self.parts[pt].pts[0]
                - tan * ((self.parts[pt].pts[3] - self.parts[pt].pts[0]).length() * 0.3);
        }
        self.parts[0].pts[1] = (self.parts[0].pts[0] + self.parts[0].pts[2]) / 2.;
        let pt = self.parts.len();
        self.parts[pt - 1].pts[2] = (self.parts[pt - 1].pts[3] + self.parts[pt - 1].pts[1]) / 2.;
    }

    pub fn compute_derivatives(&mut self) {
        if let Some(d) = &mut self.derivative {
            for (i, d) in d.iter_mut().enumerate() {
                if (self.updates[i]).is_modified() {
                    *d = self.parts[i].derivative();
                }
            }
        } else {
            self.derivative = Some(self.parts.iter().map(|c| c.derivative()).collect());
        }
    }

    pub fn create_meshes(&mut self, assets: &mut Assets<Mesh>) -> Vec<Handle<Mesh>> {
        //self.compute_derivatives();
        const STEP: f32 = 0.1;
        const ERR: f32 = 0.05;
        let mut ret = vec![];
        for (i, flag) in self.updates.iter_mut().enumerate() {
            if let Some(handle) = flag.set(assets, || {
                let walker = BezierWalker {
                    curve: &self.parts[i],
                    derivative: Cow::Owned(self.parts[i].derivative()),
                    t: 0.,
                    step_sq: STEP * STEP,
                    err_sq: ERR * ERR,
                    end: 1.,
                };
                mesh_from_curve(self.parts[i].centroid(), walker)
            }) {
                ret.push(handle);
            }
        }
        ret
    }

    pub fn insert(&mut self, pt: usize, loc: Vec3) {
        if pt > 0 {
            let new = CubicBezier::new(self.parts[pt - 1].pts[3], Vec3::ZERO, Vec3::ZERO, loc);
            self.parts.insert(pt, new);
        } else {
            let new = CubicBezier::new(self.parts[pt].pts[0], Vec3::ZERO, Vec3::ZERO, loc);
            self.parts.insert(pt, new);
        }
        if let Some(next) = self.parts.get_mut(pt + 1) {
            next.pts[0] = loc;
        }
        if pt > 0 {
            self.updates.get_mut(pt - 1).map_or((), |u| u.modified());
        }
        self.updates.insert(pt, MeshUpdate::Insert);
        self.updates.get_mut(pt + 1).map_or((), |u| u.modified());
        self.compute_tweens();
    }

    pub fn update_transforms<'a>(
        &self,
        q: impl Iterator<Item = (Mut<'a, Transform>, &'a BezierSection)>,
    ) {
        for (mut t, s) in q {
            if let Some(i) = self.updates.iter().position(|u| u.has(&s.1)) {
                t.translation = self.parts[i].centroid();
            }
        }
    }

    pub fn get_control_points(&self) -> Vec<Vec3> {
        let mut ret = vec![self.parts[0].pts[0]];
        for p in self.parts.iter() {
            ret.push(p.pts[3]);
        }
        ret
    }
}

impl<C: Bezier> Bezier for PolyBezier<C> {
    type Derivative = PolyBezier<C::Derivative>;

    /// Evaluate the curve at point t
    fn eval(&self, t: f32) -> Vec3 {
        let f = t.fract();
        let wh = t.floor();
        self.parts[wh as usize].eval(f)
    }

    fn centroid(&self) -> Vec3 {
        let mut ret = Vec3::ZERO;
        for c in self.parts.iter() {
            ret += c.centroid();
        }
        ret / self.parts.len() as f32
    }

    fn derivative(&self) -> Self::Derivative {
        PolyBezier {
            parts: self
                .derivative
                .clone()
                .unwrap_or_else(|| self.parts.iter().map(|b| b.derivative()).collect()),
            derivative: None,
            updates: vec![MeshUpdate::Insert; self.updates.len()],
        }
    }

    fn walker<'a>(&'a self, step: f32, err: f32) -> BezierWalker<'a, Self> {
        BezierWalker {
            curve: self,
            derivative: Cow::Owned(self.derivative()),
            t: 0.,
            step_sq: step * step,
            err_sq: err * err,
            end: self.parts.len() as f32,
        }
    }
}

pub struct BSplineW {
    curve: BSpline<Vec3, f32>,
    derivative: BSpline<Vec3, f32>,
}

impl BSplineW {
    pub fn new(points: Vec<Vec3>) -> Self {
        //let knots = vec![-2.0, -2.0, -2.0, -2.0, -1.0, 0.0, 1.0, 2.0, 2.0, 2.0, 2.0];
        let knots = vec![-2.0, -2.0, -1.0, -0.5, 0.5, 1.0, 2.0, 2.0];
        let degree = 3;
        let mut derivative_points = vec![];
        for i in 1..points.len() {
            derivative_points.push(
                (points[i] - points[i - 1])
                    * (degree as f32 / (knots[i + degree + 1] - knots[i + 1])),
            );
        }
        let derivative = BSpline::new(
            degree - 1,
            derivative_points,
            knots[1..knots.len() - 1].to_vec(),
        );
        let spline = BSpline::new(degree, points, knots);
        BSplineW {
            curve: spline,
            derivative,
        }
    }

    pub fn walker<'a>(&'a self, step: f32) -> BSplineWalker<'a> {
        let (cur, end) = self.curve.knot_domain();
        BSplineWalker {
            curve: self,
            cur,
            end,
            step,
        }
    }
}

pub struct BSplineWalker<'a> {
    curve: &'a BSplineW,
    cur: f32,
    end: f32,
    step: f32,
}

impl<'a> Iterator for BSplineWalker<'a> {
    type Item = CurvePoint;

    fn next(&mut self) -> Option<Self::Item> {
        self.cur += self.step;
        if self.cur < self.end {
            let point = self.curve.curve.point(self.cur);
            let up = Vec3::new(0., 0.1, 0.);
            let tangent = self.curve.derivative.point(self.cur);
            let normal = tangent.cross(up).normalize() * 0.1;
            Some(CurvePoint {
                point,
                up,
                normal,
                tangent,
                t: self.cur,
            })
        } else {
            None
        }
    }
}

//pub struct BSpline {
//pts: Vec<Vec3>,
//}

//impl BSpline {
//pub fn new(pts: Vec<Vec3>) -> Self {
//Self { pts }
//}

//pub fn eval(&self, t: f32) -> Vec3 {
//todo!()
//}

//fn get_t(t: f32, alpha: f32, p0: Vec3, p1: Vec3) -> f32 {
//let d = p1 - p0;
//t + d.length_squared().powf(alpha * 0.5)
//}

//fn catmull_rom(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
//let t0 = 0.;
//let t1 = Self::get_t(t0, 0.5, p0, p1);
//let t2 = Self::get_t(t1, 0.5, p1, p2);
//let t3 = Self::get_t(t2, 0.5, p2, p3);
//let t = t; // TODO: lerp(t1, t2, t)
//let a1 = (t1 - t) / (t1 - t0) * p0 + (t - t0) / (t1 - t0) * p1;
//let a2 = (t2 - t) / (t2 - t1) * p1 + (t - t1) / (t2 - t1) * p2;
//let a3 = (t3 - t) / (t3 - t2) * p2 + (t - t2) / (t3 - t2) * p3;
//let b1 = (t2 - t) / (t2 - t0) * a1 + (t - t0) / (t2 - t0) * a2;
//let b2 = (t3 - t) / (t3 - t1) * a2 + (t - t1) / (t3 - t1) * a3;
//let c0 = (t2 - t) / (t2 - t1) * b1 + (t - t1) / (t2 - t1) * b2;
//c0
//}
//}

//pub struct BSplineWalker<'a> {
//spline: &'a BSpline,

//}
