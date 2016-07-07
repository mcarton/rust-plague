#![feature(plugin)]
#![plugin(plague)]

plague! {
    for [()]
    test bar //~ERROR: unresolved name `bar`
}

fn main() {}
