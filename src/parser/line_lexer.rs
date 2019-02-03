use core::fmt;
use crate::parser::line_lexer::State::*;

pub const ALLOWED_PARAMETER_NAME_CHARS: &str = "-abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
pub const COMP_BEGIN_S: &str = "BEGIN";
pub const COMP_END_S: &str = "END";
pub type Pos = usize;

type StateFn = fn(&mut LineLexer) -> State;

enum State {
	Next(StateFn),
	Stop,
}

impl fmt::Debug for State {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match &self{
			Next(sfn)=> {
				f.write_str("Next(")?;
				fmt::Pointer::fmt(&(*sfn as *const ()), f)?;
				f.write_str(")")
			},
			Stop=> write!(f,"Stop")
		}

	}
}



enum Rune {
	EOF,
	Invalid,
	Valid(char),
}


#[derive(Clone, Debug)]
pub struct Item {
	pub typ: ItemType,
	pub pos: Pos,
	pub val: String,
	pub line: u32,
}

impl fmt::Display for Item {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		if let ItemType::Error = self.typ {
			write!(f, "{}", self.val)
		} else if self.val.len() > 10 {
			write!(f, "{:.10}...", self.val)
		} else {
			write!(f, "{}", self.val)
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum ItemType {
	// error occurred; value is text of error
	Error,
	// the value of a property parameter, can contain ^^, ^' or ^n
	ParamValue,
	// the value of a property, if the property is of type TEXT, the value can contain \\ , \; , \, , \n or \N
	PropValue,
	// the Property/Parameter Name
	Id,
	// an indicator for the start of a component
	Begin,
	// an indicator for the end of a component
	End,
	// the component name
	CompName,
}

#[derive(Debug)]
pub struct LineLexer {
	//current line
	pub line: (String, u32),

	// current position in the input
	pos: Pos,
	// start position of this item
	start: Pos,
	// width of last rune read from input
	width: Pos,

	//next item to be emitted
	emit: Option<Item>,

	//the next state to run
	state: State,
}

impl LineLexer {
	// lex creates a new scanner for the input string.
	pub fn new(line: u32, input: String) -> Self {
		LineLexer {
			line:(input,line),
			pos: 0,
			start: 0,
			width: 0,
			emit:None,
			state:Next(lex_prop_name),
		}
	}



	pub fn next_item(&mut self) -> Option<Item> {
		while let None=self.emit{
			match self.state {
				State::Next(sfn) => {
					self.state = sfn(self);
				},
				State::Stop => break,
			}
		}
		self.emit.take()
	}

	pub fn get_line(&self) ->(String,u32){
		return (self.line.0.clone(),self.line.1)
	}

	fn next(&mut self) -> Rune {
		if self.pos >= self.line.0.len() {
			self.width = 0;
			Rune::EOF
		} else if self.line.0.is_char_boundary(self.pos) {
			let rune = self.line.0[self.pos..].chars().next().unwrap();
			if rune == '\u{FFFD}' {
				self.width = 0;
				Rune::Invalid
			} else {
				self.width = rune.len_utf8();
				self.pos += self.width;
				Rune::Valid(rune)
			}
		} else {
			//self.pos is not at a boundary, we may have jumped into an utf8-sequence
			unreachable!("LineLexer::next (line.rs:{}):Jumped into an utf8-sequence, this should not happen!\ninput:{:?}\npos:{}", line!(), self.line.0.as_bytes(),self.pos);
			//self.width = 0;
			//Rune::Invalid
		}
	}


	fn backup(&mut self) {
		self.pos -= self.width;
	}

	fn peek(&mut self) -> Rune {
		let out = self.next();
		self.backup();
		out
	}

	fn ignore(&mut self) {
		self.start = self.pos;
	}

	fn accept(&mut self, valid: &str) -> bool {
		if let Rune::Valid(x) = self.next() {
			if valid.contains(x) {
				return true;
			}
		}
		self.backup();
		false
	}

	fn accept_run(&mut self, valid: &str) {
		while self.accept(valid) {}
		//no character found anymore, last iteration also already backed up the pos pointer.
	}

	fn accept_unless(&mut self, valid: &str) -> bool {
		if let Rune::Valid(x) = self.next() {
			//x shouldn't be in valid and shouldn't be a control char
			if !valid.contains(x) && (x >= (0x20 as char) || x == '\t') && x != (0x7f as char) {
				return true;
			}
		}
		self.backup();
		false
	}

	fn accept_run_unless(&mut self, valid: &str) {
		while self.accept_unless(valid) {}
		//no character found anymore, last iteration also already backed up the pos pointer.
	}

	fn emit(&mut self, i: ItemType) {
		self.emit=Some(
			Item {
				typ: i,
				pos: self.start,
				val: self.line.0[self.start..self.pos].to_string(),
				line: self.line.1,
			}
		);
		self.start = self.pos;
	}

	fn emit_with_trimmed_quotes(&mut self, i: ItemType) {
		let matcher: &[_] = &['"' as char];
		self.emit=Some(
			Item {
				typ: i,
				pos: self.start,
				val: self.line.0[self.start..self.pos].trim_matches(matcher).to_string(),
				line: self.line.1,
			}
		);
		self.start = self.pos;
	}
	// errorf returns an error token and terminates the scan by passing
	// back a Stop that will be the next state, closing the channel.
	fn errorf(&mut self, errstr: &str) -> State {
		self.emit=Some(
			Item {
				typ: ItemType::Error,
				pos: self.start,
				val: errstr.to_string(),
				line: self.line.1,
			}
		);
		Stop
	}

}

// lexPropName scans until a colon or a semicolon
fn lex_prop_name(l: &mut LineLexer) -> State {
	l.accept_run(ALLOWED_PARAMETER_NAME_CHARS);
	if l.pos == l.start {
		return l.errorf("expected one or more alphanumerical characters or '-'");
	}
	if l.line.0[l.start..l.pos].to_uppercase() == COMP_BEGIN_S {
		l.emit(ItemType::Begin);
		return Next(lex_before_comp_name);
	}
	if l.line.0[l.start..l.pos].to_uppercase() == COMP_END_S {
		l.emit(ItemType::End);
		return Next(lex_before_comp_name);
	}

	l.emit(ItemType::Id);
	return Next(lex_before_value);
}

fn lex_before_comp_name(l: &mut LineLexer) -> State {
	if l.accept(":") {
		l.ignore();
		return Next(lex_comp_name);
	}

	return l.errorf("expected ':'");
}

fn lex_comp_name(l: &mut LineLexer) -> State {
	if let Rune::EOF = l.peek() {
		return l.errorf("component name can't have length 0");
	}
	l.accept_run(ALLOWED_PARAMETER_NAME_CHARS);
	match l.peek() {
		Rune::EOF => {
			l.emit(ItemType::CompName);
			Stop
		}
		_ => {
			l.ignore();
			return l.errorf("unexpected character, expected eol, alphanumeric or '-'");
		}
	}
}

fn lex_before_value(l: &mut LineLexer) -> State {
	if l.accept(":") {
		l.ignore();
		return Next(lex_value);
	}
	if l.accept(";") {
		l.ignore();
		return Next(lex_param_name);
	}
	return l.errorf("expected ':' or ';'");
}

fn lex_param_name(l: &mut LineLexer) -> State {
	l.accept_run(ALLOWED_PARAMETER_NAME_CHARS);
	if l.pos == l.start {
		return l.errorf("name must not be empty");
	}
	l.emit(ItemType::Id);
	Next(lex_before_param_value)
}

fn lex_before_param_value(l: &mut LineLexer) -> State {
	if l.accept("=") {
		l.ignore();
		return Next(lex_param_value);
	}
	return l.errorf("expected '='");
}

fn lex_param_value(l: &mut LineLexer) -> State {
	if l.accept("\"") {
		return Next(lex_param_q_value);
	}
	l.accept_run_unless("\",;:");
	l.emit(ItemType::ParamValue);
	return Next(lex_after_param_value);
}

fn lex_param_q_value(l: &mut LineLexer) -> State {
	l.accept_run_unless("\"");

	if let Rune::Valid('"') = l.next() {
		l.emit_with_trimmed_quotes(ItemType::ParamValue);
		return Next(lex_after_param_value);
	}

	return l.errorf("expected '\"' or other non-control-characters");
}

fn lex_after_param_value(l: &mut LineLexer) -> State {
	if l.accept(":") {
		l.ignore(); //l.emit(itemColon)
		return Next(lex_value);
	}
	if l.accept(";") {
		l.ignore(); //l.emit(itemSemicolon)
		return Next(lex_param_name);
	}
	if l.accept(",") {
		l.ignore(); //l.emit(itemComma)
		return Next(lex_param_value);
	}
	return l.errorf("expected ',', ':' or ';'");
}

fn lex_value(l: &mut LineLexer) -> State {
	if let Rune::EOF = l.peek() {
		return l.errorf("property value can't have length 0");
	}
	l.accept_run_unless("");
	if let Rune::EOF = l.peek() {
		l.emit(ItemType::PropValue);
		return Stop;
	}
	return l.errorf("unexpected character, expected eol");
}
