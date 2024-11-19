use core::fmt::Debug;

use crate::gcode::expression::{NamedParam, Param};

pub trait EvalContext: Debug {
    fn const_fold(&self) -> bool;
    fn get_param(&self, param: &Param) -> Option<f32>;
    fn named_param_exists(&self, param: &NamedParam) -> bool;
}
