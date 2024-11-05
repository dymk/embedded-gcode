use crate::gcode::{Axes, Axis};

test_parse_axis!(x1, ["X", "1"], |b| {
    Axes::new().set(Axis::X, b.lit(1.0).clone())
});

test_parse_axis!(x1_decimal, ["X", "1.0"], |b| {
    Axes::new().set(Axis::X, b.lit(1.0).clone())
});

test_parse_axis!(x1_lower, ["x", "1"], |b| {
    Axes::new().set(Axis::X, b.lit(1.0).clone())
});

test_parse_axis!(x1_neg, ["X", "-1"], |b| {
    Axes::new().set(Axis::X, b.lit(-1.0).clone())
});

test_parse_axis!(x1_neg_decimal, ["X", "-1.0"], |b| {
    Axes::new().set(Axis::X, b.lit(-1.0).clone())
});

test_parse_axis!(x1_y2, ["X", "1.0", "Y", "2.4"], |b| {
    Axes::new()
        .set(Axis::X, b.lit(1.0).clone())
        .set(Axis::Y, b.lit(2.4).clone())
});

test_parse_axis!(x1_y2_swapped, ["Y", "2.4", "X", "1.0"], |b| {
    Axes::new()
        .set(Axis::X, b.lit(1.0).clone())
        .set(Axis::Y, b.lit(2.4).clone())
});
