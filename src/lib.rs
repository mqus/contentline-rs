//! # ContentLine-rs
//!
//! ContentLine-rs is a parser for data which is formatted in so-called contentlines.
//! This includes iCalendar and vCard, which are for exchanging calendar and contact information.
//!
//! This crate is only the backend for other parsers and provides a good abstraction over information
//! represented in contentlines.
//!
//! ## The Contentline format
//!
//! contentlines are defined in [RFC 5455] (for iCalendar) and [RFC 6350] (vCard). The general format
//! distinguishes between **Components** (which represent an entity),
//! **Properties** (which give attributes to an entity) and **Parameters**
//! (which further describe properties). Components can be a part of other Components, Properties
//! are a part of Components and Parameters are a part Properties.
//! Components have a name, which defines the type a component has and can have multiple
//! sub-components and properties. Properties have a name, which defines what property is specified,
//! and a value, which is the content of the attribute. Properties can also have multiple parameters.
//! A parameter specifies the property content further and also consists of a name, but can have one
//! or multiple values.
//! An example of a contentline-formatted file would be:
//! ```txt
//! BEGIN:COMPONENT\r\n
//! PROPERTY-NAME;PARAMETER-NAME=Parameter-value 1,Parameter-value 2:Property value\r\n
//! END:COMPONENT\r\n
//! ```
//!
//! More on this can be read in the mentioned RFCs and also [RFC 6868], which defines a parameter
//! value encoding.
//!
//! ## Encoding/Parsing
//!
//! ```
//!
//! ```
//!
//! [RFC 5455]:https://tools.ietf.org/html/rfc5545#section-3.1
//! [RFC 6350]:https://tools.ietf.org/html/rfc6350#section-3.3
//! [RFC 6868]:https://tools.ietf.org/html/rfc6868




use std::collections::HashMap;
use std::error::Error as StdError;
use core::fmt;

pub use crate::encoder::ComponentEncode;
pub use crate::encoder::Encoder;
pub use crate::parser::{Parser, rfc6868};
pub use crate::parser::Error;


mod parser;
mod encoder;

#[cfg(test)]
mod encoder_tests;

#[cfg(test)]
mod api_tests;

#[cfg(test)]
mod test_helper;


pub type Parameters = HashMap<String, Vec<String>>;

const ALLOWED_PARAMETER_NAME_CHARS: &str = "-abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const COMP_BEGIN_S: &str = "BEGIN";
const COMP_END_S: &str = "END";

#[derive(Debug)]
pub struct Property {
	pub name: String,
	pub value: String,
	pub parameters: Parameters,
	pub old_line: Option<(String, u32)>,
}

impl Property {
	pub fn new_empty(name: String, value: String) -> Result<Self, InvalidNameError> {
		Self::new(name, value, HashMap::new())
	}

	pub fn new(name: String, value: String, parameters: Parameters) -> Result<Self, InvalidNameError> {
		let x = Self { name, value, parameters, old_line: None };
		x.check()?;
		Ok(x)
	}

	pub fn check(&self) -> Result<(), InvalidNameError> {
		if let Some(c) = is_valid_name(&self.name) {
			return Err(InvalidNameError {
				typ: NameType::Property,
				violation: c,
				name: self.name.clone(),
			});
		}

		for p_name in self.parameters.keys() {
			if let Some(c) = is_valid_name(p_name) {
				return Err(InvalidNameError {
					typ: NameType::Parameter,
					violation: c,
					name: p_name.clone(),
				});
			}
		}
		Ok(())
	}

	pub fn add_param(&mut self, name: String, value: String) -> Result<(), InvalidNameError> {
		if let Some(c) = is_valid_name(name.as_str()) {
			return Err(InvalidNameError {
				typ: NameType::Parameter,
				violation: c,
				name: name.clone(),
			});
		}
		self.parameters.entry(name)
				.or_default().push(value);
		Ok(())
	}

	pub fn get_param_value(&self, name: &str) -> Option<&Vec<String>> {
		self.parameters.get(name)
	}
}

#[derive(Debug)]
pub struct Component {
	pub name: String,
	pub properties: Vec<Property>,
	pub sub_components: Vec<Component>,
}

impl Component {
	pub fn new_empty(name: String) -> Result<Self, InvalidNameError> {
		Self::new(name, Vec::new(), Vec::new())
	}

	pub fn new(name: String, properties: Vec<Property>, sub_components: Vec<Component>) -> Result<Self, InvalidNameError> {
		let c = Component { name, properties, sub_components };
		c.check()?;
		Ok(c)
	}

	pub fn check(&self) -> Result<(), InvalidNameError> {
		if let Some(c) = is_valid_name(&self.name) {
			return Err(InvalidNameError {
				typ: NameType::Component,
				violation: c,
				name: self.name.clone(),
			});
		}

		for p in &self.properties {
			p.check()?
		}
		for c in &self.sub_components {
			c.check()?
		}

		Ok(())
	}
	//MAYBE implement more API

	pub fn find_property(&self, name: &str) -> Vec<&Property> {
		let mut out = Vec::new();
		for p in &self.properties {
			if p.name == name {
				out.push(p);
			}
		}
		out
	}

	pub fn add_property(&mut self, p: Property) {
		self.properties.push(p)
	}

	pub fn add_sub_component(&mut self, c: Component) {
		self.sub_components.push(c)
	}

	pub fn encode_to_string(&self) -> String {
		let mut buf = vec![];

		//there really should not be any io errors, as the target is only memory.
		buf.encode_component(self).unwrap();

		//there should also be no utf8 encoding errors, as the input is a structure of UTF8-Strings
		// and we take care not to produce invalid characters.
		String::from_utf8(buf).unwrap()
	}
}


pub fn is_valid_name(name: &str) -> Option<char> {
	for (_, c) in name.char_indices() {
		if !ALLOWED_PARAMETER_NAME_CHARS.contains(c) {
			return Some(c);
		}
	}
	None
}

#[derive(Debug)]
pub struct InvalidNameError {
	typ: NameType,
	violation: char,
	name: String,
}

impl StdError for InvalidNameError {}

impl fmt::Display for InvalidNameError {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "{} name \"{}\" is invalid: character '{}' is not allowed", self.typ, self.name, self.violation)
	}
}

#[derive(Debug)]
enum NameType {
	Component,
	Property,
	Parameter,
}

impl fmt::Display for NameType {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		use NameType::*;
		match self {
			Component => write!(f, "component"),
			Property => write!(f, "property"),
			Parameter => write!(f, "parameter"),
		}
	}
}


