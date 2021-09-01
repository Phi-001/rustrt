#![allow(dead_code)]

use super::vector::*;
use super::Float;
use std::ops::Mul;

type Matrix = [Float; 16];

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    mat: Matrix,
    inv: Matrix,
}

impl Default for Transform {
    fn default() -> Transform {
        Transform {
            mat: identity(),
            inv: identity(),
        }
    }
}

impl Transform {
    pub fn inverse(&self) -> Transform {
        Transform {
            mat: self.inv,
            inv: self.mat,
        }
    }
    pub fn transpose(&self) -> Transform {
        Transform {
            mat: transpose(self.mat),
            inv: transpose(self.inv),
        }
    }
    pub fn translate(delta: &Vector3) -> Transform {
        Transform {
            mat: [
                1.0, 0.0, 0.0, delta.x, //
                0.0, 1.0, 0.0, delta.y, //
                0.0, 0.0, 1.0, delta.z, //
                0.0, 0.0, 0.0, 1.0, //
            ],
            inv: [
                1.0, 0.0, 0.0, -delta.x, //
                0.0, 1.0, 0.0, -delta.y, //
                0.0, 0.0, 1.0, -delta.z, //
                0.0, 0.0, 0.0, 1.0, //
            ],
        }
    }
    pub fn scale(scale: &Vector3) -> Transform {
        let inv = scale.inverse();
        Transform {
            mat: [
                scale.x, 0.0, 0.0, 0.0, //
                0.0, scale.y, 0.0, 0.0, //
                0.0, 0.0, scale.z, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ],
            inv: [
                inv.x, 0.0, 0.0, 0.0, //
                0.0, inv.y, 0.0, 0.0, //
                0.0, 0.0, inv.z, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ],
        }
    }
    pub fn rotate_x(theta: Float) -> Transform {
        let theta = theta.to_radians();
        let cos = Float::cos(theta);
        let sin = Float::sin(theta);
        let mat = [
            1.0, 0.0, 0.0, 0.0, //
            0.0, cos, -sin, 0.0, //
            0.0, sin, cos, 0.0, //
            0.0, 0.0, 0.0, 1.0, //
        ];
        Transform {
            mat,
            inv: transpose(mat),
        }
    }
    pub fn rotate_y(theta: Float) -> Transform {
        let theta = theta.to_radians();
        let cos = Float::cos(theta);
        let sin = Float::sin(theta);
        let mat = [
            cos, 0.0, sin, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            -sin, 0.0, cos, 0.0, //
            0.0, 0.0, 0.0, 1.0, //
        ];
        Transform {
            mat,
            inv: transpose(mat),
        }
    }
    pub fn rotate_z(theta: Float) -> Transform {
        let theta = theta.to_radians();
        let cos = Float::cos(theta);
        let sin = Float::sin(theta);
        let mat = [
            cos, -sin, 0.0, 0.0, //
            sin, cos, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0, //
        ];
        Transform {
            mat,
            inv: transpose(mat),
        }
    }

    // I like nicely indented stuff
    #[allow(clippy::identity_op)]
    #[allow(clippy::erasing_op)]
    pub fn look_at(look_from: &Point3, look_at: &Point3, up: &Vector3) -> Transform {
        let mut transform = identity();

        transform[0 * 4 + 3] = look_from.x;
        transform[1 * 4 + 3] = look_from.y;
        transform[2 * 4 + 3] = look_from.z;
        transform[3 * 4 + 3] = 1.0;

        let dir = Vector3::unit_vector(*look_at - *look_from);
        let right = Vector3::unit_vector(Vector3::cross(&Vector3::unit_vector(*up), &dir));
        let up = Vector3::cross(&dir, &right);
        transform[0 * 4 + 0] = right.x;
        transform[1 * 4 + 0] = right.y;
        transform[2 * 4 + 0] = right.z;
        transform[3 * 4 + 0] = 0.;
        transform[0 * 4 + 1] = up.x;
        transform[1 * 4 + 1] = up.y;
        transform[2 * 4 + 1] = up.z;
        transform[3 * 4 + 1] = 0.;
        transform[0 * 4 + 2] = dir.x;
        transform[1 * 4 + 2] = dir.y;
        transform[2 * 4 + 2] = dir.z;
        transform[3 * 4 + 2] = 0.;

        Transform {
            mat: inverse(transform),
            inv: transform,
        }
    }

    pub fn apply(&self, vector: &Vector3, w: Float) -> Vector3 {
        let Vector3 { x, y, z } = vector;

        let xp = self.mat[0] * x + self.mat[1] * y + self.mat[2] * z + self.mat[3] * w;
        let yp = self.mat[4] * x + self.mat[5] * y + self.mat[6] * z + self.mat[7] * w;
        let zp = self.mat[8] * x + self.mat[9] * y + self.mat[10] * z + self.mat[11] * w;
        let wp = self.mat[12] * x + self.mat[13] * y + self.mat[14] * z + self.mat[15] * w;

        // Is this more efficient? idk
        // Should check
        if (wp - 1.0).abs() < Float::EPSILON {
            Vector3::new(xp, yp, zp)
        } else {
            Vector3::new(xp, yp, zp) / wp
        }
    }
}

impl Mul for Transform {
    type Output = Transform;

    fn mul(self, rhs: Transform) -> Self::Output {
        Transform {
            mat: multiply(self.mat, rhs.mat),
            inv: multiply(rhs.inv, self.inv),
        }
    }
}

fn transpose(mat: Matrix) -> Matrix {
    [
        mat[0], mat[4], mat[8], mat[12], //
        mat[1], mat[5], mat[9], mat[13], //
        mat[2], mat[6], mat[10], mat[14], //
        mat[3], mat[7], mat[11], mat[15], //
    ]
}

fn identity() -> Matrix {
    [
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0, //
    ]
}

fn multiply(m1: Matrix, m2: Matrix) -> Matrix {
    let mut m = identity();
    for i in 0..4 {
        for j in 0..4 {
            m[i * 4 + j] = 0.0;
            for k in 0..4 {
                m[i * 4 + j] += m1[i * 4 + k] + m2[k * 4 + j];
            }
        }
    }

    m
}

fn inverse(mat: Matrix) -> Matrix {
    let mut index_column = [0; 4];
    let mut index_row = [0; 4];
    let mut index_pivot = [0; 4];
    let mut mat_inv = mat;
    for i in 0..4 {
        let mut row = 0;
        let mut column = 0;
        let mut big = 0.0;
        // Choose pivot
        for j in 0..4 {
            if index_pivot[j] != 1 {
                for (k, &pivot) in index_pivot.iter().enumerate() {
                    if pivot == 0 && mat_inv[j * 4 + k].abs() >= big {
                        big = mat_inv[j * 4 + k].abs();
                        row = j;
                        column = k;
                    } else if pivot > 1 {
                        panic!("Singular matrix in MatrixInvert");
                    }
                }
            }
        }
        index_pivot[column] += 1;
        if row != column {
            for k in 0..4 {
                mat_inv.swap(row * 4 + k, column * 4 + k);
            }
        }
        index_row[i] = row;
        index_column[i] = column;
        if mat_inv[column * 4 + column] == 0.0 {
            panic!("Singular matrix in MatrixInvert");
        }

        let pivot_inv = 1.0 / mat_inv[column * 4 + column];
        mat_inv[column * 4 + column] = 1.0;

        for j in 0..4 {
            mat_inv[column * 4 + j] *= pivot_inv;
        }

        // Subtract this row from others to zero out their columns
        for j in 0..4 {
            if j != column {
                let save = mat_inv[j * 4 + column];
                mat_inv[j * 4 + column] = 0.0;
                for k in 0..4 {
                    mat_inv[j * 4 + k] -= mat_inv[column * 4 + k] * save;
                }
            }
        }
    }
    // Swap columns to reflect permutation
    for j in (0..4).rev() {
        if index_row[j] != index_column[j] {
            for i in 0..4 {
                mat_inv.swap(i * 4 + index_row[j], index_column[j]);
            }
        }
    }

    mat_inv
}
