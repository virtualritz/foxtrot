use crate::{abstract_curve::AbstractCurve, nd_curve::NdBsplineCurve};
use nalgebra_glm::DVec3;

pub type BsplineCurve = NdBsplineCurve<3>;

impl AbstractCurve for BsplineCurve {
    fn point(&self, u: f64) -> DVec3 {
        self.curve_point(u)
    }
    fn derivatives<const E: usize>(&self, u: f64) -> Vec<DVec3> {
        self.curve_derivatives::<E>(u)
    }
}
