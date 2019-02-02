use std::io::Result;
use std::io::Write;

use crate::{Component, Property};
use crate::parser::{COMP_BEGIN_S, COMP_END_S};
use crate::parser::rfc6868;

const FOLDING_LENGTH: usize = 75;

pub struct Encoder<W: Write> {
	out: W
}

impl<W> Encoder<W> where W: Write {
	pub fn new(writer: W) -> Encoder<W> {
		Encoder { out: writer }
	}

	pub fn into_writer(self) -> W {
		self.out
	}


	pub fn encode(&mut self, component: &Component) -> Result<()> {
		write!(self.out, "{}:{}\r\n", COMP_BEGIN_S, component.name.to_uppercase())?;

		for prop in &component.properties {
			self.encode_property(prop)?;
		}

		for comp in &component.sub_components {
			self.encode(comp)?;
		}

		write!(self.out, "{}:{}\r\n", COMP_END_S, component.name.to_uppercase())?;
		Ok(())
	}

	pub fn encode_property(&mut self, property: &Property) -> Result<()> {
		let mut buf = vec![];

		self.write_folded(&mut buf, &property.name.to_uppercase())?;

		for (key, values) in &property.parameters {
			self.write_folded(&mut buf, ";")?;
			self.write_folded(&mut buf, key.to_uppercase().as_str())?;
			self.write_folded(&mut buf, "=")?;

			for (i, val) in values.iter().enumerate() {
				if i > 0 { self.write_folded(&mut buf, ",")?; }

				let escaped = rfc6868::escape_param_value(val);

				if escaped.contains(',') || escaped.contains(';') || escaped.contains(':') {
					self.write_folded(&mut buf, "\"")?;
					self.write_folded(&mut buf, &escaped)?;
					self.write_folded(&mut buf, "\"")?;
				} else {
					self.write_folded(&mut buf, &escaped)?;
				}
			}
		}


		self.write_folded(&mut buf, ":")?;
		self.write_folded(&mut buf, &property.value)?;
		self.out.write(buf.as_slice())?;
		write!(self.out,"\r\n")
	}

	fn write_folded(&mut self, buf: &mut Vec<u8>, mut data: &str) -> Result<()> {
		while buf.len() + data.len() > FOLDING_LENGTH {
			//dlen bytes of data can fit into the current line.
			let mut dlen = FOLDING_LENGTH - buf.len();


			//make sure not to break in the middle of utf8-sequences (not required by the standard)
			while !data.is_char_boundary(dlen) { dlen -= 1; }


			//write out the buffer, write out the allowed count of bytes and a newline character.
			self.out.write_all(buf.as_slice())?;
			self.out.write_all(data[..dlen].as_bytes())?;
			self.out.write_all(b"\r\n")?;

			//push ' ' into the empty buffer, to begin a new folded line
			buf.clear();
			buf.push(b' ');
			//set data to include only non-written data
			data = &data[dlen..];
		}
		buf.extend(data.as_bytes());
		Ok(())
	}
}

//a possibility for the future, to simplify the API even more.

//pub trait ComponentEncode{
//	fn encode_component(&mut self,component:&Component)->Result<()>;
//}
//
//impl<W> ComponentEncode for W where W:Write{
//	fn encode_component(&mut self, component: &Component) -> Result<()> {
//
//	}
//}
