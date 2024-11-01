extern crate std;

use crate::gcode::*;

test_parser!(command, g0, ["G0"], |_| Gcode::G0(None).into());

test_parser!(command, g0_x1, ["G0", "X1"], |b| Gcode::G0(Some(
    Axes::new().set(Axis::X, b.lit(1.0).clone())
))
.into());

test_parser!(command, g0_x_expr_lit, ["G0", "X", "[", "1", "]"], |b| {
    Gcode::G0(Some(Axes::new().set(Axis::X, b.lit(1.0).clone()))).into()
});

test_parser!(
    command,
    g0_x_expr_binop,
    ["G0", "X", "[", "1", "+", "2", "]"],
    |b| {
        Gcode::G0(Some(
            Axes::new().set(Axis::X, b.binop(b.lit(1.0), "+", b.lit(2.0)).clone()),
        ))
        .into()
    }
);

test_parser!(command, g20, ["G20"], |_| Gcode::G20.into());
test_parser!(command, g21, ["G21"], |_| Gcode::G21.into());
test_parser!(command, g53, ["G53"], |_| Gcode::G53.into());
test_parser!(command, g54, ["G54"], |_| Gcode::G54.into());
test_parser!(command, g90, ["G90"], |_| Gcode::G90.into());
test_parser!(command, g91, ["G91"], |_| Gcode::G91.into());

test_parser!(command, o100_if, ["O100", "if", "1"], |b| Ocode::new(
    100,
    OcodeStatement::If(b.lit(1.0).clone())
)
.into());

test_parser!(command, o100_if_lower, ["o100", "if", "1"], |b| Ocode::new(
    100,
    OcodeStatement::If(b.lit(1.0).clone())
)
.into());

test_parser!(command, o100_endif, ["o100", "endif"], |_| Ocode::new(
    100,
    OcodeStatement::EndIf
)
.into());

test_parser!(command, o100_sub, ["o100", "sub"], |_| Ocode::new(
    100,
    OcodeStatement::Sub
)
.into());

test_parser!(command, m3, ["M3"], |_| Mcode::M3.into());
test_parser!(command, m4, ["M4"], |_| Mcode::M4.into());
test_parser!(command, m5, ["M5"], |_| Mcode::M5.into());
test_parser!(command, m6, ["M6"], |_| Mcode::M6(None).into());
test_parser!(command, m6_t6, ["M6", "T8"], |_| Mcode::M6(Some(Tcode(8)))
    .into());
test_parser!(command, m7, ["M7"], |_| Mcode::M7.into());
test_parser!(command, m8, ["M8"], |_| Mcode::M8.into());
test_parser!(command, m9, ["M9"], |_| Mcode::M9.into());

test_parser!(command, s1000, ["S1000"], |_| Scode(1000.0).into());
test_parser!(command, t1, ["T1"], |_| Tcode(1).into());
