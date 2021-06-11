use nalgebra_glm::{DVec3};
use crate::{nd_curve::NDBSplineCurve, abstract_curve::AbstractCurve};

pub type BSplineCurve = NDBSplineCurve<3>;

impl AbstractCurve for BSplineCurve {
    fn point(&self, u: f64) -> DVec3 {
        self.curve_point(u)
    }
    fn derivs(&self, u: f64, d: usize) -> Vec<DVec3> {
        self.curve_derivs(u, d)
    }
}
