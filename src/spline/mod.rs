
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

pub enum CurveSegment {
    Insert,
    Mesh(Handle<Mesh>),
    MeshModified(Handle<Mesh>),
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
    ty: SplineType,
    //meshes: Vec<Handle<Mesh>>,
}

impl<C: Bezier> Clone for PolyBezier<C> {
    fn clone(&self) -> Self {
        Self {
            parts: self.parts.clone(),
            derivative: self.derivative.clone(),
            updates: vec![MeshUpdate::Insert; self.updates.len()],
            ty: self.ty,
        }
    }
}

impl PolyBezier<CubicBezier> {
    pub fn new(points: Vec<Vec3>, ty: SplineType) -> Self {
        assert!(points.len() > 1);
        if points.len() == 2 {
            Self {
                parts: vec![CubicBezier::new(points[0], points[0], points[1], points[1])],
                derivative: None,
                updates: vec![MeshUpdate::Insert],
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
                derivative: None,
                parts,
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
                mesh_from_curve(self.parts[i].centroid(), walker, self.ty)
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
            ty: self.ty,
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
