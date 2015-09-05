extern crate nalgebra;
extern crate std;

use self::nalgebra::{Vec3, Vec4, Rot3, Mat3, Mat4, Eye, Diag, Col};
use self::nalgebra::Rotation as RotationTrait;

fn extend_mat3_to_4<N: nalgebra::BaseNum>(m: Mat3<N>) -> Mat4<N> {
    Mat4 {
        m11: m.m11,     m12: m.m12,     m13: m.m13,     m14: N::zero(),
        m21: m.m21,     m22: m.m22,     m23: m.m23,     m24: N::zero(),
        m31: m.m31,     m32: m.m32,     m33: m.m33,     m34: N::zero(),
        m41: N::zero(), m42: N::zero(), m43: N::zero(), m44: N::one(),
    }
}

fn extend_vec3_to_4<N: nalgebra::BaseNum>(v: Vec3<N>, aditional: N) -> Vec4<N> {
    Vec4::new(v.x, v.y, v.z, aditional)
}

fn shrink_vec4_to_3<N: nalgebra::BaseNum>(v: Vec4<N>) -> Vec3<N> {
    Vec3::new(v.x, v.y, v.z)
}

// #[derive(Copy, Clone)]
// pub struct Transform {
//     pub s: Scale,
//     pub r: Rotation,
//     pub t: Translation,
// }

// impl Transform {
//     pub fn new() -> Transform {
//         Transform {
//             s: Scale::new(),
//             r: Rotation::new(),
//             t: Translation::new(),
//         }
//     }

//     pub fn with_rotation(mut self, rot_axis_angle: Vec3<f32>) -> Transform {
//         self.r.set_rotation(rot_axis_angle);
//         self
//     }

//     pub fn with_translation(mut self, t: Vec3<f32>) -> Transform {
//         self.t.set_translation(t);
//         self
//     }

//     pub fn with_scale(mut self, scale_factor: Factor) -> Transform {
//         self.s.set_scale(scale_factor);
//         self
//     }

//     pub fn with_rotation_mut(&mut self, rot_axis_angle: Vec3<f32>) -> &mut Transform {
//         self.r.set_rotation(rot_axis_angle);
//         self
//     }

//     pub fn with_translation_mut(&mut self, t: Vec3<f32>) -> &mut Transform {
//         self.t.set_translation(t);
//         self
//     }

//     pub fn with_scale_mut(&mut self, scale_factor: Factor) -> &mut Transform {
//         self.s.set_scale(scale_factor);
//         self
//     }

//     pub fn to_mat(&self) -> Mat4<f32> {
//         self.t.to_mat() * self.r.to_mat() * self.s.to_mat()
//     }

//     pub fn to_array(&self) -> [[f32; 4]; 4] {
//         self.to_mat().as_array().clone()
//     }
// }

// pub enum Factor {
//     Vector(Vec3<f32>),
//     Scalar(f32),
// }

// #[derive(Copy, Clone)]
// pub struct Scale {
//     scale: Mat4<f32>,
// }

// impl Scale {
//     pub fn new() -> Scale {
//         Scale {
//             scale: Mat4::new_identity(4)
//         }
//     }

//     pub fn set_scale(&mut self, factor: Factor) -> &mut Scale {
//         match factor {
//             Factor::Vector(Vec3 { x, y, z }) => {
//                 self.scale.m11 = x;
//                 self.scale.m22 = y;
//                 self.scale.m33 = z;
//             },
//             Factor::Scalar(f) => {
//                 self.scale.m11 = f;
//                 self.scale.m22 = f;
//                 self.scale.m33 = f;
//             }
//         };
//         self
//     }

//     pub fn get_scale(&self) -> Vec3<f32> {
//         Vec3::new(self.scale.m11, self.scale.m22, self.scale.m33)
//     }

//     pub fn to_mat(&self) -> Mat4<f32> {
//         self.scale
//     }
// }

#[derive(Copy, Clone)]
pub struct Rotation {
    rot: Rot3<f32>
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

    // pub fn add_rotation(&mut self, rot_axis_angle: Vec3<f32>) {
    //     self.rot.prepend_rotation_mut(&rot_axis_angle);
    // }

    pub fn to_mat(&self) -> Mat4<f32> {
        extend_mat3_to_4(self.rot.submat().clone())
    }

    // pub fn look_at(&mut self, at: Vec3<f32>, up: Vec3<f32>) {
    //     self.rot.look_at_z(&at, &up);
    // }
}

#[derive(Copy, Clone)]
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
        self.mat.set_col(3, extend_vec3_to_4(pos, 1.0));
    }

    pub fn add_translation(&mut self, pos: Vec3<f32>) {
        let cur_transl = self.get_translation();
        self.mat.set_col(3, extend_vec3_to_4(pos + cur_transl, 1.0));
    }

    pub fn get_translation(&self) -> Vec3<f32> {
        shrink_vec4_to_3(self.mat.col(3))
    }

    pub fn to_mat(&self) -> Mat4<f32> {
        self.mat
    }
}
