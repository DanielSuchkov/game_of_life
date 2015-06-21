extern crate nalgebra;
extern crate std;

use self::nalgebra::{Vec3, Vec4, Rot3, Mat3, Mat4, Eye, Diag, Col};
use self::nalgebra::Rotation as RotationTrait;

pub struct Transform {
    pub s: Scale,
    pub r: Rotation,
    pub t: Translation,
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            s: Scale::new(),
            r: Rotation::new(),
            t: Translation::new(),
        }
    }

    pub fn with_rotation(&mut self, rot_axis_angle: Vec3<f32>) -> &mut Transform {
        self.r.set_rotation(rot_axis_angle);
        self
    }

    pub fn with_translation(&mut self, t: Vec3<f32>) -> &mut Transform {
        self.t.set_translation(t);
        self
    }

    pub fn with_scale(&mut self, scale_factor: Factor) -> &mut Transform {
        self.s.set_scale(scale_factor);
        self
    }

    pub fn to_mat(&self) -> Mat4<f32> {
        self.t.to_mat() * self.r.to_mat() * self.s.to_mat()
    }

    pub fn to_array(&self) -> [[f32; 4]; 4] {
        self.to_mat().as_array().clone()
    }
}

pub enum Factor {
    Vector(Vec3<f32>),
    Scalar(f32),
}

pub struct Scale {
    scale: Mat4<f32>,
}

impl Scale {
    pub fn new() -> Scale {
        Scale {
            scale: Mat4::new_identity(4)
        }
    }

    pub fn set_scale(&mut self, factor: Factor) -> &mut Scale {
        match factor {
            Factor::Vector(Vec3 { x, y, z }) => {
                self.scale.m11 = x;
                self.scale.m22 = y;
                self.scale.m33 = z;
            },
            Factor::Scalar(f) => {
                self.scale.m11 = f;
                self.scale.m22 = f;
                self.scale.m33 = f;
            }
        };
        self
    }

    pub fn get_scale(&self) -> Vec3<f32> {
        Vec3::new(self.scale.m11, self.scale.m22, self.scale.m33)
    }

    pub fn to_mat(&self) -> Mat4<f32> {
        self.scale
    }
}

pub struct Rotation {
    rot: Rot3<f32>
}

fn extend<N: nalgebra::BaseNum>(m: Mat3<N>) -> Mat4<N> {
    Mat4 {
        m11: m.m11,     m12: m.m12,     m13: m.m13,     m14: N::zero(),
        m21: m.m21,     m22: m.m22,     m23: m.m23,     m24: N::zero(),
        m31: m.m31,     m32: m.m32,     m33: m.m33,     m34: N::zero(),
        m41: N::zero(), m42: N::zero(), m43: N::zero(), m44: N::one(),
    }
}

impl Rotation {
    pub fn new() -> Rotation {
        Rotation {
            rot: Rot3::from_diag(&Vec3::new(1.0, 1.0, 1.0))
        }
    }

    pub fn set_rotation(&mut self, rot_axis_angle: Vec3<f32>) {
        self.rot.set_rotation(rot_axis_angle);
    }

    pub fn to_mat(&self) -> Mat4<f32> {
        extend(self.rot.submat().clone())
    }
}

pub struct Translation {
    mat: Mat4<f32>
}

impl Translation {
    pub fn new() -> Translation {
        Translation {
            mat: Mat4::new_identity(4)
        }
    }

    pub fn set_translation(&mut self, pos: Vec3<f32>) {
        self.mat.set_col(3, Vec4{x: pos.x, y: pos.y, z: pos.z, w: 1.0});
    }

    pub fn to_mat(&self) -> Mat4<f32> {
        self.mat
    }

    pub fn to_array(&self) -> [[f32; 4]; 4] {
        self.mat.as_array().clone()
    }
}
