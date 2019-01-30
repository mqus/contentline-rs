use core::fmt;
use std::error::Error;

use crate::parser::lexer::Item;
use crate::parser::lexer::ItemType;
use crate::parser::lexer::LexerHandle;
use crate::parser::lexer::LexerError;

mod lexer;

struct Parser {
	lexer: Option<LexerHandle>,
	line: u32,
	r: u32,//TODO:Reader
}

impl Parser {
	//getNextItem returns the next lexer item, feeding (unfolded) lines into the lexer if neccessary.
	// It also converts identifiers (itemCompName, itemID) into upper case, errors encountered by the
	// lexer into 'error' values and property parameter values into their original value (without escaped characters).
	fn get_next_item(&mut self) -> Result<Item, Box<dyn Error>> {
		if let None = self.lexer {
			let line = self.read_unfolded_line()?;
			self.lexer = Some(lexer::new(self.line, line));
		}
		//TODO handle second unwrap (lexer eof)
		let mut i = self.lexer.as_ref().unwrap().next_item().unwrap();

		match i.typ {
			ItemType::Error => {
				let e = Box::new(
					LexerError::new(self.lexer.take().unwrap().input, i, ""));
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
		Ok(i)
	}

	fn read_unfolded_line(&mut self) -> Result<String, Box<dyn Error>> {
		unimplemented!()

		//TODO
	}
}

fn unescape_param_value(escaped: String) -> String {
	//TODO
	escaped
}

