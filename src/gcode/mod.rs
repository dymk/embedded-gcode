pub mod expression;
use alloc::string::String;
use expression::{Expression, Param};

use crate::{parser::IParseResult, NUM_AXES};

pub trait GcodeParser
where
    Self: Sized,
{
    fn parse<'i>(input: &'i [u8]) -> IParseResult<'i, Self>;
}

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Comment(String),
    Assign(Param, Expression),
    G(Gcode),
    M(Mcode),
    O(Ocode),
    S(Scode),
    T(Tcode),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Gcode {
    G0(Option<Axes>),
    G1(Axes),
    /// inch units
    G20,
    /// mm units
    G21,
    /// machine coordinates
    G53,
    /// coordinate system 1
    G54,
    /// coordinate system 2
    G55,
    /* todo - G56 -> G59.3 */
    /// absolute positioning
    G90,
    /// relative positioning
    G91,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Mcode {
    M3,
    M4,
    M5,
    M6(Option<Tcode>),
    M7,
    M8,
    M9,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Ocode {
    id: u32,
    statement: OcodeStatement,
}

impl Ocode {
    pub fn new(id: u32, statement: OcodeStatement) -> Self {
        Self { id, statement }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum OcodeStatement {
    Sub,
    EndSub,
    If(Expression),
    EndIf,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Scode(pub f32);

#[derive(Debug, PartialEq, Clone)]
pub struct Tcode(pub u32);

#[derive(Debug, PartialEq, Clone)]
pub struct Fcode(pub f32);

#[derive(Debug)]
pub enum Axis {
    X,
    Y,
    Z,
    A,
    B,
    C,
}

impl Axis {
    pub fn from_chr(chr: char) -> Option<Self> {
        match chr.to_ascii_uppercase() {
            'X' => Some(Axis::X),
            'Y' => Some(Axis::Y),
            'Z' => Some(Axis::Z),
            'A' => Some(Axis::A),
            'B' => Some(Axis::B),
            'C' => Some(Axis::C),
            _ => None,
        }
    }
    fn to_idx(&self) -> usize {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
            Axis::A => 3,
            Axis::B => 4,
            Axis::C => 5,
        }
    }
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Axes([Option<Expression>; NUM_AXES]);
impl Axes {
    pub fn new() -> Self {
        Self([const { None }; NUM_AXES])
    }
    pub fn get(&self, axis: Axis) -> Option<&Expression> {
        self.0[axis.to_idx()].as_ref()
    }
    pub fn set(mut self, axis: Axis, value: Expression) -> Self {
        self.0[axis.to_idx()] = Some(value);
        self
    }
}

macro_rules! from_impl {
    ($($name:ident $ty:ident),+) => {
        $(
            impl From<$ty> for Command {
                fn from(t: $ty) -> Self {
                    Command::$name(t)
                }
            }
        )+
    };
    () => {

    };
}

from_impl!(M Mcode, S Scode, T Tcode);

impl From<Ocode> for Command {
    fn from(t: Ocode) -> Self {
        Command::O(t)
    }
}

impl From<Gcode> for Command {
    fn from(t: Gcode) -> Self {
        Command::G(t)
    }
}
