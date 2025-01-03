use super::model_state::{ModelState, ModelStateUnit};
use crate::{
    eval::{Eval as _, EvalContext},
    gcode::{
        expression::{Expression, NamedParam, Param},
        Command, Gcode,
    },
};
use alloc::string::String;

#[derive(Debug, Default)]
pub struct Interpreter {
    local_vars_numbered: hashbrown::HashMap<u32, f32>,
    local_vars_named: hashbrown::HashMap<String, f32>,
    global_vars: hashbrown::HashMap<String, f32>,
    model_state: ModelState,
}

#[derive(Debug, PartialEq, Clone)]
pub enum InterpretError {
    ParamNotFound(Param),
    CannotEval(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub enum InterpretValue {
    EvalExpr(f32),
    Other,
}

type InterpretResult = Result<InterpretValue, InterpretError>;

impl Interpreter {
    pub fn interpret(&mut self, command: Command) -> InterpretResult {
        match command {
            Command::Comment(_) => todo!(),
            Command::Assign(to, from) => self.interpret_assign(to, from),
            Command::G(gcode) => self.interpret_gcode(gcode),
            Command::M(_) => todo!(),
            Command::O(_) => todo!(),
            Command::S(_) => todo!(),
            Command::T(_) => todo!(),
        }
    }

    fn interpret_gcode(&mut self, gcode: Gcode) -> InterpretResult {
        match gcode {
            Gcode::G20 => {
                self.model_state.selected_unit = ModelStateUnit::In;
            }
            Gcode::G21 => {
                self.model_state.selected_unit = ModelStateUnit::Mm;
            }
            _ => todo!("{:?}", gcode),
        }
        Ok(InterpretValue::Other)
    }

    pub fn get_model_state(&self) -> &ModelState {
        &self.model_state
    }

    fn interpret_assign(&mut self, to: Param, from: Expression) -> InterpretResult {
        let from = match self.eval_expr(&from) {
            Some(val) => val,
            None => return Err(InterpretError::CannotEval(from)),
        };

        let to = self
            .get_param_or_initialize_mut(&to)
            .ok_or(InterpretError::ParamNotFound(to))?;
        *to = from;
        Ok(InterpretValue::EvalExpr(from))
    }

    fn get_param_or_initialize_mut(&mut self, param: &Param) -> Option<&mut f32> {
        match param {
            Param::Numbered(num) => Some(self.get_numbered_param_or_initialize_mut(*num)),
            Param::Expr(expr) => {
                let num = match self.eval_expr(expr) {
                    Some(num) => num,
                    None => return None,
                };
                Some(self.get_numbered_param_or_initialize_mut(num as u32))
            }
            Param::NamedLocal(named_local_param) => {
                if self.local_vars_named.contains_key(named_local_param) {
                    self.local_vars_named.get_mut(named_local_param)
                } else {
                    self.local_vars_named.insert(named_local_param.clone(), 0.0);
                    self.local_vars_named.get_mut(named_local_param)
                }
            }
            Param::NamedGlobal(named_global_param) => {
                if self.global_vars.contains_key(named_global_param) {
                    self.global_vars.get_mut(named_global_param)
                } else {
                    self.global_vars.insert(named_global_param.clone(), 0.0);
                    self.global_vars.get_mut(named_global_param)
                }
            }
        }
    }

    fn get_numbered_param_or_initialize_mut(&mut self, param_num: u32) -> &mut f32 {
        self.local_vars_numbered.entry(param_num).or_insert(0.0)
    }

    fn get_param(&self, param: &Param) -> Option<f32> {
        match param {
            Param::Numbered(numbered_param) => self.get_numbered_param(*numbered_param),
            Param::NamedLocal(named_local_param) => self.get_local_param(named_local_param),
            Param::NamedGlobal(named_global_param) => self.get_global_param(named_global_param),
            Param::Expr(expr) => {
                let param_num = match self.eval_expr(expr) {
                    Some(num) => num as u32,
                    None => return None,
                };
                self.get_numbered_param(param_num)
            }
        }
    }

    pub fn get_local_param(&self, name: &str) -> Option<f32> {
        self.local_vars_named.get(name).copied()
    }
    pub fn get_global_param(&self, name: &str) -> Option<f32> {
        self.global_vars.get(name).copied()
    }
    pub fn get_numbered_param(&self, name: u32) -> Option<f32> {
        self.local_vars_numbered.get(&name).copied()
    }

    fn eval_expr(&self, expression: &Expression) -> Option<f32> {
        expression.eval(self)
    }
}

impl EvalContext for Interpreter {
    fn const_fold(&self) -> bool {
        true
    }

    fn get_param(&self, param: &Param) -> Option<f32> {
        self.get_param(param)
    }

    fn named_param_exists(&self, param: &NamedParam) -> bool {
        match param {
            NamedParam::NamedLocal(named_local_param) => {
                self.local_vars_named.contains_key(named_local_param)
            }
            NamedParam::NamedGlobal(named_global_param) => {
                self.global_vars.contains_key(named_global_param)
            }
        }
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use crate::GcodeParser;

    #[rstest::rstest]
    /* arithmetic */
    #[case("1.0", 1.0)]
    #[case("1.0 + 2.0", 3.0)]
    #[case("1.0 + 2.0 * 3.0", 7.0)]
    #[case("1.0 + 2.0 * 3.0 / 4.0", 2.5)]
    #[case("1.0 + 2.0 * 3.0 / 4.0 - 5.0", -2.5)]
    /* xor */
    #[case("1.0 XOR 0.0", 1.0)]
    #[case("0.0 XOR 1.0", 1.0)]
    #[case("1.0 XOR 1.0", 0.0)]
    #[case("0.0 XOR 0.0", 0.0)]
    /* and */
    #[case("1.0 AND 0.0", 0.0)]
    #[case("0.0 AND 1.0", 0.0)]
    #[case("1.0 AND 1.0", 1.0)]
    #[case("0.0 AND 0.0", 0.0)]
    /* or */
    #[case("1.0 OR 0.0", 1.0)]
    #[case("0.0 OR 1.0", 1.0)]
    #[case("1.0 OR 1.0", 1.0)]
    #[case("0.0 OR 0.0", 0.0)]
    /* ge */
    #[case("1.0 GE 0.0", 1.0)]
    #[case("0.0 GE 1.0", 0.0)]
    #[case("1.0 GE 1.0", 1.0)]
    #[case("0.0 GE 0.0", 1.0)]
    /* gt */
    #[case("1.0 GT 0.0", 1.0)]
    #[case("0.0 GT 1.0", 0.0)]
    #[case("1.0 GT 1.0", 0.0)]
    #[case("0.0 GT 0.0", 0.0)]
    /* le */
    #[case("1.0 LE 0.0", 0.0)]
    #[case("0.0 LE 1.0", 1.0)]
    #[case("1.0 LE 1.0", 1.0)]
    #[case("0.0 LE 0.0", 1.0)]
    /* lt */
    #[case("1.0 LT 0.0", 0.0)]
    #[case("0.0 LT 1.0", 1.0)]
    #[case("1.0 LT 1.0", 0.0)]
    #[case("0.0 LT 0.0", 0.0)]
    /* ne */
    #[case("1.0 NE 0.0", 1.0)]
    #[case("0.0 NE 1.0", 1.0)]
    #[case("1.0 NE 1.0", 0.0)]
    #[case("0.0 NE 0.0", 0.0)]
    /* eq */
    #[case("1.0 EQ 0.0", 0.0)]
    #[case("0.0 EQ 1.0", 0.0)]
    #[case("1.0 EQ 1.0", 1.0)]
    #[case("0.0 EQ 0.0", 1.0)]
    /* atan */
    #[case("ATAN[1.0]/[1.0]", 0.785_398_2)]
    #[case("ATAN[1.0]/[0.0]", 1.570_796_4)]
    #[case("ATAN[0.0]/[1.0]", 0.0)]
    #[case("ATAN[-1.0]/[1.0]", -0.785_398_2)]
    #[case("ATAN[-1.0]/[-1.0]", -2.356_194_5)]
    /* ln */
    #[case("LN[1.0]", 0.0)]
    #[case("LN[2.718281828459045]", 1.0)]
    /* sqrt */
    #[case("SQRT[1.0]", 1.0)]
    #[case("SQRT[4.0]", 2.0)]
    #[case("SQRT[9.0]", 3.0)]
    /* sin */
    #[case("SIN[0.0]", 0.0)]
    #[case("SIN[1.5707963267948966]", 1.0)]
    /* cos */
    #[case("COS[0.0]", 1.0)]
    #[case("COS[1.5707963267948966]", 0.0)]
    /* tan */
    #[case("TAN[0.0]", 0.0)]
    /* exp */
    #[case("EXP[0.0]", 1.0)]
    #[case("EXP[1.0]", std::f32::consts::E)]
    /* abs */
    #[case("ABS[1.0]", 1.0)]
    #[case("ABS[-1.0]", 1.0)]
    /* acos */
    #[case("ACOS[1.0]", 0.0)]
    #[case("ACOS[0.0]", 1.570_796_4)]
    /* asin */
    #[case("ASIN[1.0]", 1.570_796_4)]
    #[case("ASIN[0.0]", 0.0)]
    /* round */
    #[case("ROUND[1.0]", 1.0)]
    #[case("ROUND[1.4]", 1.0)]
    #[case("ROUND[1.5]", 2.0)]
    #[case("ROUND[-1.4]", -1.0)]
    #[case("ROUND[-1.5]", -2.0)]
    /* exists */
    #[case("EXISTS[#<x>]", 0.0)]
    #[case("EXISTS[#<y>]", 0.0)]
    #[case("EXISTS[#<_y>]", 0.0)]
    /* fix */
    #[case("FIX[1.4]", 1.0)]
    #[case("FIX[1.5]", 1.0)]
    #[case("FIX[1.6]", 1.0)]
    #[case("FIX[-1.4]", -2.0)]
    #[case("FIX[-1.5]", -2.0)]
    #[case("FIX[-1.6]", -2.0)]
    /* fup */
    #[case("FUP[1.4]", 2.0)]
    #[case("FUP[1.5]", 2.0)]
    #[case("FUP[1.6]", 2.0)]
    #[case("FUP[-1.4]", -1.0)]
    #[case("FUP[-1.5]", -1.0)]
    #[case("FUP[-1.6]", -1.0)]
    /* pow */
    #[case("2.0 ** 3.0", 8.0)]
    #[case("2.0 ** 0.0", 1.0)]
    #[case("2.0 ** -1.0", 0.5)]
    #[case("-2.0 ** 3.0", -8.0)]
    #[case("-2.0 ** 2.0", 4.0)]
    #[case("-2.0 ** -1.0", -0.5)]
    fn test_eval_expr(#[case] input: &str, #[case] expected: f32) {
        use crate::parser::Input;

        let interpreter = Interpreter::default();
        let input = Input::new(input.as_bytes(), &interpreter);
        let expression = Expression::parse(input).unwrap().1;
        let actual = interpreter.eval_expr(&expression).unwrap();
        assert!(
            (actual - expected).abs() < 1e-6,
            "{} => {} != {}",
            input,
            actual,
            expected
        );
    }
}
