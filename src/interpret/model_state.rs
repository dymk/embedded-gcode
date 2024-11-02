use core::fmt::Debug;

use crate::NUM_AXES;

#[derive(Debug, Default)]
pub struct ModelState {
    pub selected_unit: ModelStateUnit,
    pub feedrate: MmSec,
    pub workspace: Workspace,
    pub abs_position: Position<NUM_AXES>,
}

#[derive(Debug, Default)]
struct MmSec(pub f64);

#[derive(Debug)]
struct Position<const N: usize>([f64; N]);
impl<const N: usize> Default for Position<N> {
    fn default() -> Self {
        Self([0.0; N])
    }
}

#[derive(Debug, Default)]
pub enum Workspace {
    #[default]
    Machine,
    G54,
    G55,
    G56,
    G57,
    G58,
    G59,
    G59_1,
    G59_2,
    G59_3,
}

#[derive(Debug, Default, PartialEq)]
pub enum ModelStateUnit {
    In,
    #[default]
    Mm,
}
