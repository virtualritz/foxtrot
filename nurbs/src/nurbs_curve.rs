use crate::{abstract_curve::AbstractCurve, nd_curve::NdBsplineCurve};
use nalgebra_glm::DVec3;

pub type NurbsCurve = NdBsplineCurve<4>;

impl AbstractCurve for NurbsCurve {
    /// Converts a point at position t onto the 3D line, using basis functions
    /// of order `p + 1` respectively.
    ///
    /// Algorithm A4.1
    fn point(&self, u: f64) -> DVec3 {
        let p = self.curve_point(u);
        p.xyz() / p.w
    }

    /// Computes the derivatives of the curve of order up to and including `d` at location `t`,
    /// using basis functions of order `p + 1` respectively.
    ///
    /// Algorithm A4.2
    fn derivatives<const E: usize>(&self, u: f64) -> Vec<DVec3> {
        let derivatives = self.curve_derivatives::<E>(u);
        let mut CK = vec![DVec3::zeros(); E + 1];
        for k in 0..=E {
            let mut v = derivatives[k].xyz();
            for i in 1..=k {
                let b = num_integer::binomial(k, i);
                v -= b as f64 * derivatives[i].w * CK[k - 1];
            }
            CK[k] = v / derivatives[0].w;
        }
        CK
    }
}
