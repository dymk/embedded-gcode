extern crate std;

use crate::gcode::*;

use super::TestContext;

test_parse_command!(g0, ["G0"], |_| Gcode::G0(None));

test_parse_command!(g0_x1, ["G0", "X1"], |b| Gcode::G0(Some(
    Axes::new().set(Axis::X, b.lit(1.0))
)));

test_parse_command!(g0_x_expr_lit, ["G0", "X", "[", "1", "]"], |b| {
    Gcode::G0(Some(Axes::new().set(Axis::X, b.lit(1.0))))
});

test_parse_command!(g0_x_expr_binop, ["G0", "X", "[", "1", "+", "2", "]"], |b| {
    Gcode::G0(Some(
        Axes::new().set(Axis::X, b.binop(b.lit(1.0), "+", b.lit(2.0))),
    ))
});

test_parse_command!(g1, ["G1", "X1"], |b| {
    Gcode::G1(Axes::new().set(Axis::X, b.lit(1.0)))
});

test_parse_command!(g20, ["G20"], |_| Gcode::G20);
test_parse_command!(g21, ["G21"], |_| Gcode::G21);
test_parse_command!(g53, ["G53"], |_| Gcode::G53);
test_parse_command!(g54, ["G54"], |_| Gcode::G54);
test_parse_command!(g90, ["G90"], |_| Gcode::G90);
test_parse_command!(g91, ["G91"], |_| Gcode::G91);

test_parse_command!(o100_if, ["O100", "if", "1"], |b| Ocode::new(
    100,
    OcodeStatement::If(b.lit(1.0).clone())
));

test_parse_command!(o100_if_lower, ["o100", "if", "1"], |b| Ocode::new(
    100,
    OcodeStatement::If(b.lit(1.0).clone())
));

test_parse_command!(o100_endif, ["o100", "endif"], |_| Ocode::new(
    100,
    OcodeStatement::EndIf
));

test_parse_command!(o100_sub, ["o100", "sub"], |_| Ocode::new(
    100,
    OcodeStatement::Sub
));

test_parse_command!(m3, ["M3"], |_| Mcode::M3);
test_parse_command!(m4, ["M4"], |_| Mcode::M4);
test_parse_command!(m5, ["M5"], |_| Mcode::M5);
test_parse_command!(m6, ["M6"], |_| Mcode::M6(None));
test_parse_command!(m6_t6, ["M6", "T8"], |b| Mcode::M6(Some(Tcode(b.lit(8.0)))));
test_parse_command!(m7, ["M7"], |_| Mcode::M7);
test_parse_command!(m8, ["M8"], |_| Mcode::M8);
test_parse_command!(m9, ["M9"], |_| Mcode::M9);

test_parse_command!(s1000, ["S1000"], |b| Scode(b.lit(1000.0)));
test_parse_command!(t1, ["T1"], |b| Tcode(b.lit(1.0)));

test_parse_command!(assign, ["#1", "=", "1"], |b| {
    Command::assign(b.num_param(1), b.lit(1.0))
});

test_parse_command!(assign_expr, ["#1", "=", "[", "1", "+", "2", "]"], |b| {
    Command::assign(b.num_param(1), b.binop(b.lit(1.0), "+", b.lit(2.0)))
});

test_parse_command!(assign_expr_named_local, ["#<x>", "=", "1"], |b| {
    Command::assign(b.local_param("x"), b.lit(1.0))
});

test_parse_command!(assign_expr_named_global, ["#<_y>", "=", "1"], |b| {
    Command::assign(b.global_param("_y"), b.lit(1.0))
});

test_parse_command!(
    cf_assign,
    TestContext::default().set_global("_y", 5.0),
    ["#<_y>", "=", "1"],
    |b| { Command::assign(b.global_param("_y"), b.lit(1.0)) }
);
