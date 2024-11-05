use super::*;

test_parse_param!(num_param, ["#1"], |_| { Param::Numbered(NumberedParam(1)) });

test_parse_param!(local_param, ["#<a>"], |_| {
    Param::NamedLocal(NamedLocalParam("a".to_string()))
});

test_parse_param!(global_param, ["#<_a>"], |_| {
    Param::NamedGlobal(NamedGlobalParam("_a".to_string()))
});

test_parse_param!(global_param_upper, ["#<_A>"], |_| {
    Param::NamedGlobal(NamedGlobalParam("_a".to_string()))
});
