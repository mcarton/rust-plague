#![feature(plugin)]
#![plugin(plague)]

plague! {
    for [()]
    test bar //~ERROR: error: unresolved name `bar`
}

fn main() {}
