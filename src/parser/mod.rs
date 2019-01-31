use core::fmt;
use std::error::Error;
use std::io::BufRead;
use std::io::Split;
use std::iter::Peekable;

use crate::parser::lexer::Item;
use crate::parser::lexer::ItemType;
use crate::parser::lexer::LexerError;
use crate::parser::lexer::LexerHandle;

mod lexer;

struct Parser<R: BufRead> {
	lexer: Option<LexerHandle>,
	line: u32,
	r: Peekable<Split<R>>,//TODO:Reader
}

impl<R> Parser<R> where R: BufRead {





	//getNextItem returns the next lexer item, feeding (unfolded) lines into the lexer if neccessary.
	// It also converts identifiers (itemCompName, itemID) into upper case, errors encountered by the
	// lexer into 'error' values and property parameter values into their original value (without escaped characters).
	fn get_next_item(&mut self) -> Result<Option<Item>, Box<dyn Error>> {
		if let None = self.lexer {
			if let Some(line) = self.read_unfolded_line()? {
				self.lexer = Some(lexer::new(self.line, String::from_utf8(line)?));
			} else {
				//Reached EOF
				return Ok(None);
			}
		}
		//TODO handle second unwrap (lexer eof)
		let mut i = self.lexer.as_ref().unwrap().next_item().unwrap();

		match i.typ {
			ItemType::Error => {
				let e = Box::new(
					LexerError::new(self.lexer.take().unwrap(), i, ""));
				self.lexer = None;
				return Err(e);
			}
			ItemType::CompName => {
				i.val = i.val.to_uppercase();
				self.lexer = None;
			}
			ItemType::Id => i.val = i.val.to_uppercase(),
			ItemType::PropValue => self.lexer = None,
			ItemType::ParamValue => i.val = unescape_param_value(i.val),
			_ => {}
		}
		Ok(Some(i))
	}

	fn read_unfolded_line(&mut self) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
		let mut buf;

		match self.r.next() {
			None => return Ok(None), //Reached EOF
			Some(line) => buf = line?,
		}

		// all lines have to end with a \r\n.
		if let Some(byte) = buf.pop() {
			if byte != b'\r'{
				return Err(Box::new(ParseError {
					msg: format!("Expected CRLF:{1}, >{0:?}<",buf[buf.len() - 1] as char,String::from_utf8(buf)?)
				}));
			}
		}
		buf.pop();//remove \r


		// peek at next line. If next line begins with a space or HTAB (\t), 'unfold' it.
		// if it would throw an error, don't return it (it's only borrowed), force a recursive call which will also trigger it.
		// if there is nothing to read or the next line begins with another character, don't unfold.
		let append = match self.r.peek() {
			Some(x) => match x {
				Ok(next_line) => next_line[0] == b' ' || next_line[0] == b'\t',
				Err(_) => true,
			},
			None => false,
		};

		if append {
			let mut next=self.read_unfolded_line()?.unwrap();
			//remove space/htab at the front and append
			buf.extend_from_slice(&next[1..]);
		}
		Ok(Some(buf))

	}
}

fn unescape_param_value(escaped: String) -> String {
	//TODO
	escaped
}


#[derive(Debug)]
struct ParseError {
	msg: String,
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "{}", self.msg.as_str())
	}
}


