mod bool_to_float;
mod eval_context;

pub use bool_to_float::bool_to_float;
pub use eval_context::EvalContext;

pub trait Eval {
    fn eval(&self, context: &dyn EvalContext) -> Option<f32>;
}
