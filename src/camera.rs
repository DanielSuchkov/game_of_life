extern crate nalgebra;

use nalgebra::{Vec3, Mat4, PerspMat3};

use transform::{Translation, Rotation};

#[derive(Clone, Copy)]
pub struct PerspectiveCamera {
    projection: PerspMat3<f32>,
    translation: Translation,
    rotation: Rotation,
}

impl PerspectiveCamera {
    pub fn new() -> PerspectiveCamera {
        PerspectiveCamera {
            projection: PerspMat3::new(1024.0 / 768.0, 90.0f64.to_radians() as f32, 0.1, 2000.0),
            translation: Translation::new(),
            rotation: Rotation::new(),
        }
    }

    // pub fn with_fov_mut(&mut self, deg_angle: i32) -> &mut PerspectiveCamera {
    //     self.projection.set_fov((deg_angle as f64).to_radians() as f32);
    //     self
    // }

    // pub fn with_aspect_mut(&mut self, aspect: f32) -> &mut PerspectiveCamera {
    //     self.projection.set_aspect(aspect);
    //     self
    // }

    pub fn with_view_dimensions_mut(&mut self, width: u32, height: u32) -> &mut PerspectiveCamera {
        self.projection.set_aspect(width as f32 / height as f32);
        self
    }

    // pub fn with_znear_mut(&mut self, val: f32) -> &mut PerspectiveCamera {
    //     self.projection.set_znear(val);
    //     self
    // }

    // pub fn with_zfar_mut(&mut self, val: f32) -> &mut PerspectiveCamera {
    //     self.projection.set_zfar(val);
    //     self
    // }

    pub fn with_rotation_mut(&mut self, rot: Vec3<f32>) -> &mut PerspectiveCamera {
        self.rotation.set_rotation(rot);
        self
    }

    // pub fn with_look_at_mut(&mut self, at: Vec3<f32>, up: Vec3<f32>) -> &mut PerspectiveCamera {
    //     self.rotation.look_at(at, up);
    //     self
    // }

    pub fn with_position_mut(&mut self, pos: Vec3<f32>) -> &mut PerspectiveCamera {
        self.translation.set_translation(pos);
        self
    }

    pub fn with_fov(mut self, deg_angle: i32) -> PerspectiveCamera {
        self.projection.set_fov((deg_angle as f64).to_radians() as f32);
        self
    }

    // pub fn with_aspect(mut self, aspect: f32) -> PerspectiveCamera {
    //     self.projection.set_aspect(aspect);
    //     self
    // }

    // pub fn with_view_dimensions(mut self, width: u32, height: u32) -> PerspectiveCamera {
    //     self.projection.set_aspect(width as f32 / height as f32);
    //     self
    // }

    // pub fn with_znear(mut self, val: f32) -> PerspectiveCamera {
    //     self.projection.set_znear(val);
    //     self
    // }

    // pub fn with_zfar(mut self, val: f32) -> PerspectiveCamera {
    //     self.projection.set_zfar(val);
    //     self
    // }

    // pub fn with_rotation(mut self, rot: Vec3<f32>) -> PerspectiveCamera {
    //     self.rotation.set_rotation(rot);
    //     self
    // }

    // pub fn with_look_at(mut self, at: Vec3<f32>, up: Vec3<f32>) -> PerspectiveCamera {
    //     self.rotation.look_at(at, up);
    //     self
    // }

    pub fn with_position(mut self, pos: Vec3<f32>) -> PerspectiveCamera {
        self.translation.set_translation(pos);
        self
    }

    pub fn to_vp_mat(&self) -> Mat4<f32> {
        self.projection.to_mat() * self.rotation.to_mat() * self.translation.to_mat()
    }

    pub fn to_vp_array(&self) -> [[f32; 4]; 4] {
        self.to_vp_mat().as_array().clone()
    }

    // pub fn set_position(&mut self, pos: Vec3<f32>) {
    //     self.with_position(pos);
    // }

    pub fn add_position(&mut self, pos: Vec3<f32>) {
        self.translation.add_translation(pos);
    }

    // pub fn add_rotation(&mut self, rot: Vec3<f32>) {
    //     self.rotation.add_rotation(rot);
    // }
}
