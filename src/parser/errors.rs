use std::error::Error;
use core::fmt;
use crate::parser::line_lexer::{Item,LineLexer};

#[derive(Debug)]
pub struct LexerError {
	ctx: String,
	msg: String,
	i: Item,
	line: u32,
}

const ERROR_CONTEXT_RADIUS: usize = 20;

impl LexerError {
	pub fn new(ctx: LineLexer, i: Item, msg: &str, line: u32) -> Self {
		LexerError {
			ctx: ctx.line.0,
			msg: msg.to_string(),
			i,
			line,
		}
	}
}

impl Error for LexerError {}

impl fmt::Display for LexerError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut msg = self.msg.as_str();
		let ctx = self.ctx.as_str();

		let pos1 = self.i.pos;
		let mut pos2 = self.i.pos + self.i.val.len();

		if msg == "" {
			msg = self.i.val.as_str();
			pos2 = pos1 + 1;
		}

		let prefix=if pos1 > ERROR_CONTEXT_RADIUS {
			"...".to_owned() + &ctx[pos1 - ERROR_CONTEXT_RADIUS..pos1]
		} else {
			ctx[..pos1].to_owned()
		};

		let suffix = if ctx.len() > ERROR_CONTEXT_RADIUS + pos2 {
			ctx[pos1..pos2 + ERROR_CONTEXT_RADIUS].to_owned() + "..."
		} else if pos1 < ctx.len() {
			ctx[pos1..].to_owned()
		} else {
			"".to_string()
		};

		return match suffix.len() {
			0 => writeln!(f, "{}: \t{}: {}<HERE>", self.line, msg, &prefix),
			1 => writeln!(f, "{}: \t{}: {} >{}<", self.line, msg, &prefix, &suffix[..pos2 - pos1]),
			_ => writeln!(f, "{}: \t{}: {} >{}< {}", self.line, msg, &prefix, &suffix[..pos2 - pos1], &suffix[pos2 - pos1..]),
		};
	}
}
