use crate::gcode::{Axes, Axis};

use super::TestContext;

test_parse_axes!(x1, ["X", "1"], |b| { Axes::new().set(Axis::X, b.lit(1.0)) });

test_parse_axes!(x1_decimal, ["X", "1.0"], |b| {
    Axes::new().set(Axis::X, b.lit(1.0))
});

test_parse_axes!(x1_lower, ["x", "1"], |b| {
    Axes::new().set(Axis::X, b.lit(1.0))
});

test_parse_axes!(x1_neg, ["X", "-1"], |b| {
    Axes::new().set(Axis::X, b.lit(-1.0))
});

test_parse_axes!(x1_neg_decimal, ["X", "-1.0"], |b| {
    Axes::new().set(Axis::X, b.lit(-1.0))
});

test_parse_axes!(x1_y2, ["X", "1.0", "Y", "2.4"], |b| {
    Axes::new()
        .set(Axis::X, b.lit(1.0))
        .set(Axis::Y, b.lit(2.4))
});

test_parse_axes!(x1_y2_swapped, ["Y", "2.4", "X", "1.0"], |b| {
    Axes::new()
        .set(Axis::X, b.lit(1.0))
        .set(Axis::Y, b.lit(2.4))
});

test_parse_axes!(
    x1_y5_expr,
    ["X", "1.0", "Y", "[", "2", "+", "3", "]"],
    |b| {
        Axes::new()
            .set(Axis::X, b.lit(1.0))
            .set(Axis::Y, b.binop(b.lit(2.0), "+", b.lit(3.0)))
    }
);

test_parse_axes!(
    x1_y5_expr_cf,
    TestContext::default().const_fold(true),
    ["X", "1.0", "Y", "[", "2", "+", "3", "]"],
    |b| {
        Axes::new()
            .set(Axis::X, b.lit(1.0))
            .set(Axis::Y, b.lit(5.0))
    }
);

test_parse_axes!(
    x1_param,
    TestContext::default().const_fold(true).set_numbered(1, 4.0),
    ["X", "#1"],
    |b| { Axes::new().set(Axis::X, b.lit(4.0)) }
);
