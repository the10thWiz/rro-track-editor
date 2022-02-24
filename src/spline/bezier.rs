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
use crate::gvas::SplineType;


use crate::spline_mesh::*;

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
