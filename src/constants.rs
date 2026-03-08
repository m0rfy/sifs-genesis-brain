//! SIFS Constants (BRAIN_CONTRACT, core.py)

use std::f32::consts::PI;

/// K = 1/π² ≈ 0.10132 — безразмерная константа затухания
pub const K: f32 = 1.0 / (PI * PI);
/// φ = (1+√5)/2 — золотое сечение
pub const PHI: f32 = 1.6180339887;
pub const SIFS_LEVELS: usize = 10;
/// FIB — индексы 10 S-уровней (как в core.py)
#[allow(dead_code)]
pub const FIB: [u32; SIFS_LEVELS] = [1, 2, 3, 5, 8, 13, 21, 34, 55, 89];
