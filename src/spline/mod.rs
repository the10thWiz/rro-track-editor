use std::borrow::Cow;

use crate::{gvas::SplineType, control::DefaultAssets};
use bevy::prelude::*;

mod bezier;
pub use bezier::CubicBezier;

pub mod mesh;
use mesh::*;

// TODO: Fix
#[derive(Debug, Component)]
pub struct BezierSection(usize, pub Handle<Mesh>);

pub struct CurvePoint {
    //points: [Vec3; 4],
    pub point: Vec3,
    pub up: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub t: f32,
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
pub enum MeshUpdate {
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

    #[allow(unused)]
    pub fn is_modified(&self) -> bool {
        match self {
            Self::None(_) => false,
            _ => true,
        }
    }

    pub fn set(
        &mut self,
        assets: &mut Assets<Mesh>,
        f: impl FnOnce(&Assets<Mesh>) -> Option<Mesh>,
    ) -> Option<Handle<Mesh>> {
        match self {
            Self::Insert => {
                if let Some(m) = f(assets) {
                    let mesh = assets.add(m);
                    *self = Self::None(mesh.clone_weak());
                    Some(mesh)
                } else {
                    None
                }
            }
            Self::Modified(old) => {
                if let Some(m) = f(assets) {
                    let mesh = assets.set(old.clone(), m);
                    *self = Self::None(mesh.clone_weak());
                    None
                } else {
                    None
                }
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

#[derive(Debug, Component)]
pub struct PolyBezier<C: Bezier> {
    parts: Vec<C>,
    updates: Vec<MeshUpdate>,
    visibility: Vec<bool>,
    ty: SplineType,
    //meshes: Vec<Handle<Mesh>>,
}

impl<C: Bezier> Clone for PolyBezier<C> {
    fn clone(&self) -> Self {
        Self {
            parts: self.parts.clone(),
            updates: vec![MeshUpdate::Insert; self.updates.len()],
            visibility: self.visibility.clone(),
            ty: self.ty,
        }
    }
}

impl PolyBezier<CubicBezier> {
    pub fn new(points: Vec<Vec3>, visibility: Vec<bool>, ty: SplineType) -> Self {
        assert!(points.len() > 1);
        if points.len() == 2 {
            Self {
                parts: vec![CubicBezier::new(points[0], points[0], points[1], points[1])],
                updates: vec![MeshUpdate::Insert],
                visibility,
                ty,
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
                parts,
                visibility,
                ty,
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

    pub fn create_meshes(
        &mut self,
        meshes: &mut Assets<Mesh>,
        default_assets: &Res<DefaultAssets>,
    ) -> Vec<(Handle<Mesh>, bool)> {
        //self.compute_derivatives();
        // const STEP: f32 = 0.1;
        // const ERR: f32 = 0.05;
        let mut ret = vec![];
        for (i, flag) in self.updates.iter_mut().enumerate() {
            if let Some(handle) = flag.set(meshes, |assets| {
                let mesh = default_assets.spline_mesh[self.ty].clone();
                if let Some(mesh) = assets.get(mesh) {
                    Some(mesh_on_curve(
                        mesh,
                        self.parts[i].centroid(),
                        &self.parts[i],
                    ))
                } else {
                    None
                }
            }) {
                ret.push((handle, self.visibility[i]));
            }
        }
        ret
    }

    pub fn insert(&mut self, pt: usize, loc: Vec3) {
        if pt == 0 {
            // At beginning
            self.parts
                .insert(0, CubicBezier::new_ends(loc, self.get_control_point(0)));
            self.updates.get_mut(0).map(|m| m.modified());
            self.updates.get_mut(1).map(|m| m.modified());
            self.updates.insert(0, MeshUpdate::Insert);
            self.visibility.insert(0, true);
        } else if pt == self.len() {
            // At end
            self.parts.insert(
                pt - 1,
                CubicBezier::new_ends(self.get_control_point(pt - 1), loc),
            );
            self.updates.get_mut(pt - 2).map(|m| m.modified());
            self.updates.insert(pt - 1, MeshUpdate::Insert);
            self.visibility.insert(pt - 1, true);
        } else {
            let before = self.get_control_point(pt - 1);
            self.parts[pt - 1].pts[0] = loc;
            self.parts
                .insert(pt - 1, CubicBezier::new_ends(before, loc));
            self.updates
                .get_mut(pt.saturating_sub(2))
                .map(|m| m.modified());
            self.updates.get_mut(pt - 1).map(|m| m.modified());
            self.updates.insert(pt - 1, MeshUpdate::Insert);
            self.visibility.insert(pt - 1, true);
        }
        self.compute_tweens();
    }

    pub fn before(&self, pt: usize, loc: Vec3) -> bool {
        if pt == 0 {
            let new = self.get_control_point(pt) - loc;
            let cur = self.get_control_point(pt) - self.get_control_point(pt + 1);
            if new.dot(cur) < 0. {
                true
            } else {
                false
            }
        } else {
            let new = self.get_control_point(pt) - loc;
            let cur = self.get_control_point(pt) - self.get_control_point(pt - 1);
            if new.dot(cur) < 0. {
                false
            } else {
                true
            }
        }
    }

    pub fn set_ty(&mut self, ty: SplineType) {
        self.ty = ty;
        self.updates.iter_mut().for_each(|m| m.modified());
    }

    pub fn get_transforms<'s>(&'s self) -> impl Iterator<Item = (Vec3, &MeshUpdate)> + 's {
        self.parts
            .iter()
            .map(|p| p.centroid())
            .zip(self.updates.iter())
    }

    pub fn split_pt(&self, pt: usize) -> (Self, Self) {
        let end = pt.saturating_sub(1);
        (
            Self {
                parts: Vec::from_iter(
                    self.parts
                        .get(..end)
                        .iter()
                        .flat_map(|a| a.iter())
                        .cloned(),
                ),
                updates: Vec::from_iter(
                    self.parts
                        .get(..end)
                        .iter()
                        .flat_map(|a| a.iter())
                        .map(|_| MeshUpdate::Insert),
                ),
                visibility: Vec::from_iter(
                    self.visibility
                        .get(..end)
                        .iter()
                        .flat_map(|a| a.iter())
                        .copied(),
                ),
                ty: self.ty,
            },
            Self {
                parts: Vec::from_iter(
                    self.parts
                        .get(pt + 1..)
                        .iter()
                        .flat_map(|a| a.iter())
                        .cloned(),
                ),
                updates: Vec::from_iter(
                    self.parts
                        .get(pt + 1..)
                        .iter()
                        .flat_map(|a| a.iter())
                        .map(|_| MeshUpdate::Insert),
                ),
                visibility: Vec::from_iter(
                    self.visibility
                        .get(pt + 1..)
                        .iter()
                        .flat_map(|a| a.iter())
                        .copied(),
                ),
                ty: self.ty,
            },
        )
    }

    pub fn split_sec(&self, section: &Handle<Mesh>) -> (Self, Self) {
        let pt = self.updates.iter().position(|m| m.has(section)).unwrap();
        (
            Self {
                parts: Vec::from_iter(self.parts.get(..pt).iter().flat_map(|a| a.iter()).cloned()),
                updates: Vec::from_iter(
                    self.parts
                        .get(..pt)
                        .iter()
                        .flat_map(|a| a.iter())
                        .map(|_| MeshUpdate::Insert),
                ),
                visibility: Vec::from_iter(
                    self.visibility
                        .get(..pt)
                        .iter()
                        .flat_map(|a| a.iter())
                        .copied(),
                ),
                ty: self.ty,
            },
            Self {
                parts: Vec::from_iter(
                    self.parts
                        .get(pt + 1..)
                        .iter()
                        .flat_map(|a| a.iter())
                        .cloned(),
                ),
                updates: Vec::from_iter(
                    self.parts
                        .get(pt + 1..)
                        .iter()
                        .flat_map(|a| a.iter())
                        .map(|_| MeshUpdate::Insert),
                ),
                visibility: Vec::from_iter(
                    self.visibility
                        .get(pt + 1..)
                        .iter()
                        .flat_map(|a| a.iter())
                        .copied(),
                ),
                ty: self.ty,
            },
        )
    }

    // pub fn update_transforms<'a>(
    //     &self,
    //     q: impl Iterator<Item = (Mut<'a, Transform>, &'a BezierSection)>,
    // ) {
    //     for (mut t, s) in q {
    //         if let Some(i) = self.updates.iter().position(|u| u.has(&s.1)) {
    //             t.translation = self.parts[i].centroid();
    //         }
    //     }
    // }

    pub fn get_control_points<'s>(&'s self) -> ControlPointIter<'s> {
        ControlPointIter { curve: self, i: 0 }
    }

    pub fn len(&self) -> usize {
        self.parts.len() + 1
    }

    pub fn get_control_point(&self, i: usize) -> Vec3 {
        if i == 0 {
            self.parts[0].pts[0]
        } else {
            self.parts[i - 1].pts[3]
        }
    }

    pub fn ty(&self) -> SplineType {
        self.ty
    }

    pub fn get_segment(&self, segment: &Handle<Mesh>) -> Option<usize> {
        self.updates.iter().position(|m| m.has(segment))
    }

    pub fn segment_visible(&self, segment: &Handle<Mesh>) -> bool {
        if let Some(i) = self.updates.iter().position(|m| m.has(segment)) {
            self.visibility[i]
        } else {
            false
        }
    }

    pub fn toggle_segment_visible(&mut self, segment: &Handle<Mesh>) -> bool {
        if let Some(i) = self.updates.iter().position(|m| m.has(segment)) {
            self.visibility[i] = !self.visibility[i];
            self.visibility[i]
        } else {
            false
        }
    }

    pub fn segment_modified(&self, i: usize) -> bool {
        self.updates[i].is_modified()
    }

    #[allow(unused)]
    pub fn get_modified(&self) -> Vec<bool> {
        self.updates.iter().map(|m| m.is_modified()).collect()
    }
}

pub struct ControlPointIter<'a> {
    curve: &'a PolyBezier<CubicBezier>,
    i: usize,
}

impl<'a> Iterator for ControlPointIter<'a> {
    type Item = Vec3;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.curve.len() {
            let ret = self.curve.get_control_point(self.i);
            self.i += 1;
            Some(ret)
        } else {
            None
        }
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
            parts: self.parts.iter().map(|b| b.derivative()).collect(),
            updates: vec![MeshUpdate::Insert; self.updates.len()],
            visibility: self.visibility.clone(),
            ty: self.ty,
        }
    }

    // fn walker<'a>(&'a self, step: f32, err: f32) -> BezierWalker<'a, Self> {
    //     BezierWalker {
    //         curve: self,
    //         derivative: Cow::Owned(self.derivative()),
    //         t: 0.,
    //         step_sq: step * step,
    //         err_sq: err * err,
    //         end: self.parts.len() as f32,
    //     }
    // }
}
