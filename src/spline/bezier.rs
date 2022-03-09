//
// curve.rs
// Copyright (C) 2022 matthew <matthew@matthew-ubuntu>
// Distributed under terms of the MIT license.
//

use std::borrow::Cow;

use bevy::{ecs::system::EntityCommands, prelude::*, render::mesh::Indices};
use super::{Bezier, CurvePoint};
// use bevy_transform_gizmo::TransformGizmoEvent;
// use bspline::BSpline;

// use crate::BezierSection;
// use crate::gvas::SplineType;

// use crate::spline_mesh::*;

#[derive(Debug, Clone)]
pub struct CubicBezier {
    pub pts: [Vec3; 4],
}

impl CubicBezier {
    pub fn new(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> Self {
        Self { pts: [a, b, c, d] }
    }

    pub fn update(&mut self, pt: usize, loc: Vec3) {
        self.pts[pt] = loc;
    }

    pub fn get_pts(&self) -> &[Vec3; 4] {
        &self.pts
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
