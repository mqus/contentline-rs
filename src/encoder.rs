#![allow(clippy::write_with_newline)]
use std::io::Result;
use std::io::Write;

use crate::{Component, Property};
use crate::parser::rfc6868;
use crate::{COMP_END_S,COMP_BEGIN_S};

const FOLDING_LENGTH: usize = 75;

pub struct Encoder<W: Write> {
	out: W
}

impl<W> Encoder<W> where W: Write {
	pub fn new(writer: W) -> Encoder<W> {
		Encoder { out: writer }
	}

	pub fn encode(&mut self, component: &Component) -> Result<()> {
		self.out.encode_component(component)
	}

	pub fn encode_all(&mut self, component: &[Component]) -> Result<()> {
		for c in component {
			self.out.encode_component(c)?
		}
		Ok(())
	}

	pub fn encode_into_writer(writer: &mut W, component: &Component) -> Result<()> {
		writer.encode_component(component)
	}
}

pub trait ComponentEncode {
	fn encode_component(&mut self, component: &Component) -> Result<()>;
}

impl<W> ComponentEncode for W where W: Write {
	fn encode_component(&mut self, component: &Component) -> Result<()> {
		write!(self, "{}:{}\r\n", COMP_BEGIN_S, component.name.to_uppercase())?;

		for prop in &component.properties {
			encode_property(self, prop)?;
		}

		for comp in &component.sub_components {
			self.encode_component(comp)?;
		}

		write!(self, "{}:{}\r\n", COMP_END_S, component.name.to_uppercase())?;
		Ok(())
	}
}

fn encode_property<W: Write>(writer: &mut W, property: &Property) -> Result<()> {
	let mut buf = vec![];

	write_folded(writer, &mut buf, &property.name.to_uppercase())?;

	for (key, values) in &property.parameters {
		write_folded(writer, &mut buf, ";")?;
		write_folded(writer, &mut buf, key.to_uppercase().as_str())?;
		write_folded(writer, &mut buf, "=")?;

		for (i, val) in values.iter().enumerate() {
			if i > 0 { write_folded(writer, &mut buf, ",")?; }

			let escaped = rfc6868::escape_param_value(val);

			if escaped.contains(',') || escaped.contains(';') || escaped.contains(':') {
				write_folded(writer, &mut buf, "\"")?;
				write_folded(writer, &mut buf, &escaped)?;
				write_folded(writer, &mut buf, "\"")?;
			} else {
				write_folded(writer, &mut buf, &escaped)?;
			}
		}
	}


	write_folded(writer, &mut buf, ":")?;
	write_folded(writer, &mut buf, &property.value)?;
	writer.write_all(buf.as_slice())?;
	write!(writer, "\r\n")
}

fn write_folded<W: Write>(writer: &mut W, buf: &mut Vec<u8>, mut data: &str) -> Result<()> {
	while buf.len() + data.len() > FOLDING_LENGTH {
		//dlen bytes of data can fit into the current line.
		let mut dlen = FOLDING_LENGTH - buf.len();


		//make sure not to break in the middle of utf8-sequences (not required by the standard)
		while !data.is_char_boundary(dlen) { dlen -= 1; }


		//write out the buffer, write out the allowed count of bytes and a newline character.
		writer.write_all(buf.as_slice())?;
		writer.write_all(data[..dlen].as_bytes())?;
		writer.write_all(b"\r\n")?;

		//push ' ' into the empty buffer, to begin a new folded line
		buf.clear();
		buf.push(b' ');
		//set data to include only non-written data
		data = &data[dlen..];
	}
	buf.extend(data.as_bytes());
	Ok(())
}

impl<W> From<W> for Encoder<W> where W:Write{
	fn from(x: W) -> Self {
		Encoder::new(x)
	}
}
//impl<W> From<Encoder<W>> for W where W:Write{
//	fn from(w: Encoder<W>) -> Self {
//		w.out
//	}
//}
