use std::io::{BufRead,BufReader,Read,Split};

use core::iter::Peekable;

use crate::{Component, Parameters, Property};
pub use crate::parser::errors::Error;
use crate::parser::line_lexer::{Item,ItemType,LineLexer};


mod line_lexer;
mod errors;
pub mod rfc6868;

#[cfg(test)]
mod tests;

pub struct Parser<R: BufRead> {
	lexer: Option<LineLexer>,
	line: (String,u32),
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
			line: (String::new(),0),
			next_line: 1,
			r: input.split(b'\n').peekable(),
		}
	}


	pub fn next_component(&mut self) -> Result<Option<Component>, Error> {
		self.lexer=None;
		match self.get_next_item()? {
			None => Ok(None), //EOF
			Some(i) => match i.typ {
				ItemType::Begin => Ok(Some(self.parse_component()?)),
				ItemType::End | ItemType::Id  => Err(Error::new(i,"expected BEGIN".to_string(),self.line.clone())),
				_ => unreachable!("parser::next_component: unexpected item type '{:?}' in line {}: {}", i.typ, self.line.1, i.val)
			}
		}
	}


	//parseComponent parses the Component for which itemBegin was already read.
	fn parse_component(&mut self) -> Result<Component, Error> {
		let name = match self.get_next_item()? {
			Some(i) =>
				if i.typ == ItemType::CompName {
					i.val
				} else {
					unreachable!("parser::parse_component: unexpected item type '{:?}' in line {}: {}", i.typ, self.line.1, i.val)
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
				None => return Err(Error::eof_error( out.name,self.line.1)),
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
						Ok(out)
					} else {
						Err(Error::new(item, format!("expected \"END:{}\"", out.name), self.line.clone()))
					}
				} else {
					unreachable!("unexpected item type in parser::parse_component")
				}
			}
			None => unreachable!("unexpected EOF in parser::parse_component"),
		}
	}

	//parseProperty parses the next Property while already having parsed the Property name.
	fn parse_property(&mut self, name: String) -> Result<Property, Error> {
		let mut out = Property {
			name,
			value: "".to_string(),
			parameters: Parameters::new(),
			old_line: Some(self.line.clone()),
		};
		let mut last_param_name = "".to_string();
		loop {
			match self.get_next_item()? {
				Some(item) => match item.typ {
					ItemType::Id => last_param_name = item.val,
					ItemType::ParamValue => out.add_param(last_param_name.clone(), item.val).unwrap(),
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
	fn get_next_item(&mut self) -> Result<Option<Item>, Error> {
		if self.lexer.is_none() {
			self.line.1 = self.next_line;
			if let Some(line) = self.read_unfolded_line()? {
				self.line.0 = String::from_utf8(line)?;
				self.lexer = Some(LineLexer::new(self.line.0.clone()));
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
				return Err(Error::new(i,  "".to_string(), self.line.clone()));
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

	fn read_unfolded_line(&mut self) -> Result<Option<Vec<u8>>, Error> {
		let mut buf;

		match self.r.next() {
			None => return Ok(None), //Reached EOF
			Some(line) => buf = line?,
		}
		// increment line counter
		self.next_line += 1;
		// all lines have to end with a \r\n. Empty lines without a \r\n are also not allowed. empty lines at the end return EOF (represented as Ok(None)
		if let Some(byte) = buf.pop() {
			if byte != b'\r' {
				buf.push(byte);
				return Err(Error::crlf_error(buf, self.line.1, self.r.peek().is_some()));
			}
		} else if self.r.peek().is_some() {
			//there are some lines following and this line contains only "\n" => is not allowed!
			//buf is empty (otherwise a pop() would have given Some(_), not None
			return Err(Error::crlf_error( buf, self.line.1, self.r.peek().is_some()));
		} else {

			//this is the last line (after \r\n) and it is empty.
			return Ok(None);
		}

		// peek at next line. If next line begins with a space or HTAB (\t), 'unfold' it.
		// if it would throw an error, don't return it (it's only borrowed), force a recursive call which will also trigger it.
		// if there is nothing to read or the next line begins with another character, don't unfold.
		let append = match self.r.peek() {
			Some(x) => match x {
				Err(_) => true,
				Ok(next_line) if !next_line.is_empty() => next_line[0] == b' ' || next_line[0] == b'\t',
				//there is a next line but it is empty.
				_ =>false,
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
	type Item = Result<Component, Error>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.next_component() {
			Ok(None) => None,
			Ok(Some(c)) => Some(Ok(c)),
			Err(e) => Some(Err(e)),
		}
	}
}



