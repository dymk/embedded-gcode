use super::*;
use crate::gcode::ArithmeticBinOp;

test_parse_param!(num_param, ["#1"], |_| Param::numbered(1));
test_parse_param!(num_param_indirect, ["##1"], |_| Param::expr(
    Param::numbered(1)
));

test_parse_param!(num_param_from_named, ["##<a>"], |_| Param::expr(
    Param::named_local("a")
));

test_parse_param!(num_param_expr_named, ["#[", "1", "+", "#<a>", "]"], |_| {
    Param::expr(Expression::binop(
        ArithmeticBinOp::Add,
        1.0,
        Param::named_local("a"),
    ))
});

test_parse_param!(expr_param, ["#[", "1", "+", "2", "]"], |_| Param::expr(
    Expression::binop(ArithmeticBinOp::Add, 1.0, 2.0)
));

test_parse_param!(num_param_with_spaces, ["#", "1"], |_| Param::numbered(1));

test_parse_param!(named_param_with_spaces, ["#", "<", "a", ">"], |_| {
    Param::named_local("a")
});

test_parse_param!(local_param, ["#<", "a", ">"], |_| Param::named_local("a"));

test_parse_param!(
    global_param,
    ["#<", "_", "a", ">"],
    |_| Param::named_global("_a")
);
test_parse_param!(global_param_upper, ["#<", "_", "A", ">"], |_| {
    Param::named_global("_a")
});
test_parse_param!(
    global_param_upper_spaces,
    ["#<_", "A", "b", "C", ">"],
    |_| Param::named_global("_abc")
);
