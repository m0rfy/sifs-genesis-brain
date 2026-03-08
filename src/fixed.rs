//! Fixed-point math and SIFS W(n), threshold

use crate::constants::{K, PHI};

pub type Fixed16 = i32;
pub const FIXED_ONE: Fixed16 = 1 << 16;

pub fn f32_to_fixed(x: f32) -> Fixed16 {
    (x * FIXED_ONE as f32) as Fixed16
}

pub fn fixed_to_f32(x: Fixed16) -> f32 {
    x as f32 / FIXED_ONE as f32
}

pub fn sifs_w(n: usize) -> Fixed16 {
    let w = (-2.0 * K * n as f32).exp();
    f32_to_fixed(w)
}

pub fn sifs_threshold(n: usize, v0: Fixed16) -> Fixed16 {
    let phi_n = PHI.powi(n as i32);
    (v0 as f64 / phi_n as f64) as Fixed16
}
