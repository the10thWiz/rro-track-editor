
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
