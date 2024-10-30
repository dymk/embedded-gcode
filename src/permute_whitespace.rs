extern crate std;
use std::{collections::HashSet, prelude::v1::*};

pub fn permute_whitespace(tokens: &[&str]) -> Vec<String> {
    let tokens = [&[""], tokens, &[""]].concat();
    let mut results = Vec::new();

    for i in 0..(1 << (tokens.len() - 1)) {
        let mut input = Vec::new();
        for j in 0..tokens.len() {
            input.push(tokens[j]);
            if j < tokens.len() - 1 && (i & (1 << j)) != 0 {
                input.push(" ");
            }
        }
        results.push(input.concat());
    }

    results
}

#[test]
fn test_permute_whitespace() {
    let tokens = ["foo"];
    let expected = HashSet::from(["foo", " foo", "foo ", " foo "].map(|s| s.to_string()));
    let actual: HashSet<_> = permute_whitespace(&tokens).into_iter().collect();
    assert_eq!(expected, actual);
}
