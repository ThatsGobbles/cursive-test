#![feature(clamp)]

pub mod examples;

// use examples::custom as x;
// use examples::progress as x;
// use examples::slider as x;
// use examples::tcp_server as x;
use examples::canvas as x;

fn main() {
    x::run();
}
