#![feature(plugin)]
#![plugin(plague)]

fn pos(haystack: &[u64], needle: u64) -> Option<usize> {
    // so many problems here
    for i in 1..haystack.len() {
        if haystack[i] == needle {
            return Some(i);
        }
    }

    None
}

plague! {
    for [
        (&[], 42) -> None,
        (&[42], 42) -> Some(0),
        (&[42, 1337], 42) -> Some(0),
        (&[42, 1337, 42], 42) -> Some(0),
        (&[1337, 42], 42) -> Some(1),
        (&[0, 1, 2, 3, 4, 5, 42], 42) -> Some(6),
        (&[0, 1, 2, 3, 4, 5, 6], 42) -> None,
    ]
    test pos
}

#[test]
fn without_plague() {
    let tests : &[(&[_], _, _)] = &[
        (&[], 42, None),
        (&[42], 42, None),
        (&[42, 1337], 42, Some(0)),
        (&[42, 1337, 42], 42, Some(0)),
        (&[1337, 42], 42, Some(1)),
        (&[0, 1, 2, 3, 4, 5, 42], 42, Some(6)),
        (&[0, 1, 2, 3, 4, 5, 6], 42, None),
    ];

    for &(haystack, needle, ret) in tests {
        assert_eq!(pos(haystack, needle), ret);
    }
}

fn main() {}
