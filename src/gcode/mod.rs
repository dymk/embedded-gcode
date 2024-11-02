pub mod expression;
use expression::{Expression, Param};

use crate::NUM_AXES;

#[derive(Debug, PartialEq, Clone)]
pub enum Command<'b> {
    Comment(&'b str),
    Assign(Param<'b>, Expression<'b>),
    G(Gcode<'b>),
    M(Mcode),
    O(Ocode<'b>),
    S(Scode),
    T(Tcode),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Gcode<'b> {
    G0(Option<Axes<'b>>),
    G1(Axes<'b>),
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
pub struct Ocode<'b> {
    id: u32,
    statement: OcodeStatement<'b>,
}

impl<'b> Ocode<'b> {
    pub fn new(id: u32, statement: OcodeStatement<'b>) -> Self {
        Self { id, statement }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum OcodeStatement<'b> {
    Sub,
    EndSub,
    If(Expression<'b>),
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
pub struct Axes<'b>([Option<Expression<'b>>; NUM_AXES]);
impl<'b> Axes<'b> {
    pub fn new() -> Self {
        Self([const { None }; NUM_AXES])
    }
    pub fn get(&'b self, axis: Axis) -> Option<&'b Expression<'b>> {
        self.0[axis.to_idx()].as_ref()
    }
    pub fn set(mut self, axis: Axis, value: Expression<'b>) -> Self {
        self.0[axis.to_idx()] = Some(value);
        self
    }
}

macro_rules! from_impl {
    ($($name:ident $ty:ident),+) => {
        $(
            impl From<$ty> for Command<'_> {
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

impl<'b> From<Ocode<'b>> for Command<'b> {
    fn from(t: Ocode<'b>) -> Self {
        Command::O(t)
    }
}

impl<'b> From<Gcode<'b>> for Command<'b> {
    fn from(t: Gcode<'b>) -> Self {
        Command::G(t)
    }
}
