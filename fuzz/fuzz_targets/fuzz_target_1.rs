
#![no_main]
#[macro_use] extern crate libfuzzer_sys;
use std::io::Cursor;
extern crate contentline;
use contentline::Parser;

fuzz_target!(|data: &[u8]| {
    let p=Parser::new(Cursor::new(data));
	for _obj in p{};
});
