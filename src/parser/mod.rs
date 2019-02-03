use core::fmt;
use std::error::Error;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Split;
use std::iter::Peekable;

use crate::{Component, Parameters, Property};
use crate::parser::errors::LexerError;
use crate::parser::line_lexer::{Item,ItemType,LineLexer};


mod line_lexer;
mod errors;
pub mod rfc6868;

#[cfg(test)]
mod tests;

pub struct Parser<R: BufRead> {
	lexer: Option<LineLexer>,
	line: u32,
	next_line: u32,
	r: Peekable<Split<R>>,
}

impl<R> Parser<BufReader<R>> where R: Read {
	pub fn from_unbuffered(input: R) -> Self {
		Self::new(BufReader::new(input))
	}
}


impl<R> Parser<R> where R: BufRead {
	pub fn new(input: R) -> Self {
		Parser {
			lexer: None,
			line: 0,
			next_line: 1,
			r: input.split(b'\n').peekable(),
		}
	}


	pub fn next_component(&mut self) -> Result<Option<Component>, Box<dyn Error>> {
		match self.get_next_item()? {
			None => Ok(None), //EOF
			Some(i) => match i.typ {
				ItemType::Begin => Ok(Some(self.parse_component()?)),
				ItemType::End => Err(Box::new(ParseError {
					msg: format!("unexpected END , expected BEGIN")
				})),
				ItemType::Id => Err(Box::new(ParseError {
					msg: format!("unexpected identifier, expected BEGIN")
				})),
				_ => unreachable!("parser::next_component: unexpected item type '{:?}' in line {}: {}", i.typ, self.line, i.val)
			}
		}
	}


	//parseComponent parses the Component for which itemBegin was already read.
	fn parse_component(&mut self) -> Result<Component, Box<dyn Error>> {
		let name = match self.get_next_item()? {
			Some(i) =>
				if i.typ == ItemType::CompName {
					i.val
				} else {
					unreachable!("parser::parse_component: unexpected item type '{:?}' in line {}: {}", i.typ, self.line, i.val)
				},
			None => unreachable!("unexpected EOF in parser::parse_component"),
		};

		let mut out = Component {
			name,
			properties: vec![],
			sub_components: vec![],
		};
		loop {
			match self.get_next_item()? {
				None => return Err(Box::new(ParseError {
					msg: format!("unexpected end of input, expected END:{}", out.name)
				})),
				Some(i) => match i.typ {
					ItemType::Begin => out.sub_components.push(self.parse_component()?),
					ItemType::Id => out.properties.push(self.parse_property(i.val)?),
					ItemType::End => break,
					_ => unreachable!("unexpected item type in parser::parse_component"),
				}
			}
		}
		match self.get_next_item()? {
			Some(item) => {
				if item.typ == ItemType::CompName {
					if item.val == out.name {
						return Ok(out);
					} else {
						return Err(Box::new(ParseError {
							msg: format!("expected END:{}, got END:{}", out.name, item.val)
						}));
					}
				} else {
					unreachable!("unexpected item type in parser::parse_component")
				}
			}
			None => unreachable!("unexpected EOF in parser::parse_component"),
		}
	}

	//parseProperty parses the next Property while already having parsed the Property name.
	fn parse_property(&mut self, name: String) -> Result<Property, Box<dyn Error>> {
		let mut out = Property {
			name,
			value: "".to_string(),
			parameters: Parameters::new(),
			old_line: Some(self.lexer.as_ref().unwrap().get_line()),
		};
		let mut last_param_name = "".to_string();
		loop {
			match self.get_next_item()? {
				Some(item) => match item.typ {
					ItemType::Id => last_param_name = item.val,
					ItemType::ParamValue => out.add_param(last_param_name.clone(), item.val),
					ItemType::PropValue => {
						out.value = item.val;
						return Ok(out);
					}
					_ => unreachable!("unexpected item type in parser::parse_property"),
				}
				None => unreachable!("unexpected EOF in parser::parse_property"),
			}
		}
	}

	//getNextItem returns the next lexer item, feeding (unfolded) lines into the lexer if neccessary.
	// It also converts identifiers (itemCompName, itemID) into upper case, errors encountered by the
	// lexer into 'error' values and property parameter values into their original value (without escaped characters).
	fn get_next_item(&mut self) -> Result<Option<Item>, Box<dyn Error>> {
		if let None = self.lexer {
			self.line = self.next_line;
			if let Some(line) = self.read_unfolded_line()? {
				self.lexer = Some(LineLexer::new(self.line, String::from_utf8(line)?));
			} else {
				//Reached EOF
				return Ok(None);
			}
		}

		let mut i;
		match self.lexer.as_mut().unwrap().next_item() {
			None => {
				//unexpected Tokenstream EOF, should not happen (because lexer throws an error because that happens)
				unreachable!("unexpected token stream EOF in parser::get_next_item");
				//self.l=None;
				//return Ok(None);
			}
			Some(it) => i = it,
		}

		match i.typ {
			ItemType::Error => {
				return Err(Box::new(
					LexerError::new(self.lexer.take().unwrap(), i, "", self.line)
				));
			}
			ItemType::CompName => {
				i.val = i.val.to_uppercase();
				self.lexer = None;
			}
			ItemType::Id => i.val = i.val.to_uppercase(),
			ItemType::PropValue => self.lexer = None,
			ItemType::ParamValue => i.val = rfc6868::unescape_param_value(&i.val),
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
		// increment line counter
		self.next_line += 1;

		// all lines have to end with a \r\n.
		if let Some(byte) = buf.pop() {
			if byte != b'\r' {
				return Err(Box::new(ParseError {
					msg: format!("Expected CRLF:{1}, >{0:?}<", buf[buf.len() - 1] as char, String::from_utf8(buf)?)
				}));
			}
		}

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
			let next = self.read_unfolded_line()?.unwrap();
			//remove space/htab at the front and append
			buf.extend_from_slice(&next[1..]);
		}
		Ok(Some(buf))
	}
}

impl<R> Iterator for Parser<R>
	where R: BufRead {
	type Item = Result<Component, Box<dyn Error>>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.next_component() {
			Ok(None) => None,
			Ok(Some(c)) => Some(Ok(c)),
			Err(e) => Some(Err(e)),
		}
	}
}


#[derive(Debug)]
pub struct ParseError {
	msg: String,
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "{}", self.msg.as_str())
	}
}



