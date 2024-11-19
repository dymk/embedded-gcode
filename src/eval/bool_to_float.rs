#[inline(always)]
pub fn bool_to_float(expr: bool) -> f32 {
    if expr {
        1.0
    } else {
        0.0
    }
}
