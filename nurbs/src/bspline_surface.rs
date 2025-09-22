use crate::{VecF, abstract_surface::AbstractSurface, nd_surface::NdBsplineSurface};
use nalgebra_glm::{DVec2, DVec3};

pub type BsplineSurface = NdBsplineSurface<3>;

impl AbstractSurface for BsplineSurface {
    fn point(&self, uv: DVec2) -> DVec3 {
        self.surface_point(uv)
    }

    fn point_from_basis(&self, uspan: usize, Nu: &VecF, vspan: usize, Nv: &VecF) -> DVec3 {
        self.surface_point_from_basis(uspan, Nu, vspan, Nv)
    }

    fn derivatives<const E: usize>(&self, uv: DVec2) -> Vec<Vec<DVec3>> {
        self.surface_derivatives::<E>(uv)
    }
}
