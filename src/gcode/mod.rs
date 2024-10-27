use expression::Expression;

pub mod expression;

#[derive(Debug, PartialEq, Clone)]
pub enum Command<'b> {
    Comment(&'b str),
    G(Gcode),
    M(Mcode),
    O(Ocode<'b>),
    S(Scode),
    T(Tcode),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Gcode {
    G0(Option<Axes>),
    G1(Axes),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Mcode {}

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
pub enum Scode {}

#[derive(Debug, PartialEq, Clone)]
pub enum Tcode {}

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
        match chr {
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
pub struct Axes([Option<f32>; 6]);
impl Axes {
    pub fn new() -> Self {
        Self([None; 6])
    }
    pub fn get(&self, axis: Axis) -> Option<f32> {
        self.0[axis.to_idx()]
    }
    pub fn set(mut self, axis: Axis, value: f32) -> Self {
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

from_impl!(G Gcode, M Mcode, S Scode, T Tcode);

impl<'b> From<Ocode<'b>> for Command<'b> {
    fn from(t: Ocode<'b>) -> Self {
        Command::O(t)
    }
}
