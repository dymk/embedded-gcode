use super::*;

test_parse_param!(num_param, ["#1"], |_| Param::numbered(1));

test_parse_param!(local_param, ["#<a>"], |_| Param::named_local("a"));
test_parse_param!(global_param, ["#<_a>"], |_| Param::named_global("_a"));
test_parse_param!(global_param_upper, ["#<_A>"], |_| Param::named_global("_a"));
test_parse_param!(
    global_param_upper_spaces,
    ["#<_", "A", "b", "C", ">"],
    |_| Param::named_global("_abc")
);
