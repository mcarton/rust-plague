#![feature(plugin)]
#![plugin(plague)]

plague! {
    for [] //~ERROR: empty parametrized tests are useless
    test fn empty() {
    }
}

fn main() {}
