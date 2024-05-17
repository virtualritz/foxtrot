#![allow(clippy::needless_range_loop)]
// This crate is translations of algorithms from the 70s, which use awkward
// single-character names everywhere, so we're matching their convention.
#![allow(non_snake_case)]

mod abstract_curve;
mod abstract_surface;
mod bspline_curve;
mod bspline_surface;
mod knot_vector;
mod nd_curve;
mod nd_surface;
mod nurbs_curve;
mod nurbs_surface;
mod sampled_curve;
mod sampled_surface;

use smallvec::SmallVec;
type VecF = SmallVec<[f64; 8]>;

pub use crate::abstract_curve::AbstractCurve;
pub use crate::abstract_surface::AbstractSurface;
pub use crate::bspline_curve::BsplineCurve;
pub use crate::bspline_surface::BsplineSurface;
pub use crate::knot_vector::KnotVector;
pub use crate::nd_curve::NdBsplineCurve;
pub use crate::nd_surface::NdBsplineSurface;
pub use crate::nurbs_curve::NurbsCurve;
pub use crate::nurbs_surface::NurbsSurface;
pub use crate::sampled_curve::SampledCurve;
pub use crate::sampled_surface::SampledSurface;
