use core::fmt;
use core::fmt::Display;
use std::error::Error as StdError;
use std::io;
use std::string::FromUtf8Error;

use crate::parser::errors::Error::*;
use crate::parser::line_lexer::Item;

#[derive(Debug)]
pub enum Error {
	Malformed(String, String, usize, (String, u32)),
	CRLF(Vec<u8>, u32, bool),
	UnexpectedEOF(String, u32),
	Io(io::Error),
	Utf8(FromUtf8Error),
}

impl Error {
	pub fn new(i: Item, msg: String, line: (String, u32)) -> Self {
		Malformed(msg, i.val, i.pos, line)
	}
	pub fn crlf_error(bytes: Vec<u8>, linenum: u32, has_next: bool) -> Self {
		CRLF(bytes, linenum, has_next)
	}

	pub fn eof_error(msg: String, l: u32) -> Self {
		UnexpectedEOF(msg, l)
	}
}

impl From<io::Error> for Error {
	fn from(e: io::Error) -> Self {
		Io(e)
	}
}

impl From<FromUtf8Error> for Error {
	fn from(e: FromUtf8Error) -> Self {
		Utf8(e)
	}
}

impl StdError for Error {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Malformed(..) | CRLF(..) | UnexpectedEOF(..) => None,
			Io(e) => Some(e),
			Utf8(e) => Some(e),
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match self {
			Io(e) => (e as &Display).fmt(f),
			Utf8(e) => (e as &Display).fmt(f),
			CRLF(buf, line, _) => {
				if buf.is_empty() {
					write!(f, "line {}: expected CR ('\\r') before LF in empty line", line)
				} else {
					match String::from_utf8(buf.clone()) {
						Err(e) => (&e as &Display).fmt(f),
						Ok(s) => error_msg(f, (s, *line), "expected CR ('\\r') before LF", buf.len(), buf.len()),
					}
				}
			}
			Malformed(msg, val, pos, line) => {
				if msg.is_empty() {
					//take length of char at pos or 0
					let x=line.0[*pos..].chars().next().map_or(0,|c|c.len_utf8());
					error_msg(f, line.clone(), val.as_str(), *pos, pos + x)
				} else {
					error_msg(f, line.clone(), msg.as_str(), *pos, pos + val.len())
				}
			}
			UnexpectedEOF(comp_name, line) => {
				write!(f, "line {}: Unexpected end of file or stream, expected END:{}", line, comp_name)
			}
		}
	}
}

const ERROR_CONTEXT_RADIUS: usize = 20;

fn error_msg(f: &mut fmt::Formatter, line: (String, u32), msg: &str, pos1: usize, pos2: usize) -> Result<(), fmt::Error> {
	let ctx = line.0.as_str();
	let prefix = if pos1 > ERROR_CONTEXT_RADIUS {
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
	let len = pos2-pos1;

	match suffix.len() {
		0 => writeln!(f, "line {}: \t{}: {}<HERE>", line.1, msg, prefix),
		1 => writeln!(f, "line {}: \t{}: {} >{}<", line.1, msg, prefix, &suffix[..len]),
		_ => writeln!(f, "line {}: \t{}: {} >{}< {}", line.1, msg, prefix, &suffix[..len], &suffix[len..]),
	}
}
