use crate::parser::lexer::*;
use std::sync::mpsc::Sender;

const WSP: &str = " 	";
const ALLOWED_PARAMETER_NAME_CHARS: &str = "-abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const COMP_BEGIN_S: &str = "BEGIN";
const COMP_END_S: &str = "END";

enum Rune {
	EOF,
	Invalid,
	Valid(char),
}

pub struct Lexer {
	// documented for error messages
	line: u32,
	// the string being scanned
	input: String,
	// current position in the input
	pos: Pos,
	// start position of this item
	start: Pos,
	// width of last rune read from input
	width: Pos,
	// channel of scanned items
	//item_receiver: Receiver<Item>,
	item_sender: Sender<Item>,
}


type StateFn = fn(&mut Lexer) -> Option<State>;
type StateFn2 = fn(&mut Lexer) -> NextState;

enum NextState{
	State(StateFn2),
	Stop
}

//Workaround, sadly we can't put StateFn directly into the Option.
macro_rules! state {
	($f:ident) => (Some(State{inner:Self::$f}))
}
struct State {
	inner: StateFn
}

impl Lexer {
	pub fn new(line:u32,input:String,item_sender:Sender<Item>)->Self{
		Lexer {
			line,
			input,
			pos: 0,
			start: 0,
			width: 0,
			item_sender,
		}
	}




	fn next(&mut self) -> Rune {
		if self.pos >= self.input.len() {
			self.width = 0;
			Rune::EOF
		} else if self.input.is_char_boundary(self.pos) {
			//TODO chars() could panic if codepoint seems valid but isn't
			//MAYBE does work if self.input is a lossyly decoded string (errors are replaced with 0FFFD)
			//or MAYBE copy UTF8_CHAR_WIDTH from libcore/str/mod.rs

			//make the choice between error and 0xFFFD available via the Lexer constructor
			let rune = self.input[self.pos..].chars().next().unwrap();
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
			self.width = 0;
			Rune::Invalid
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

	fn emit(&mut self, i: ItemType) {
		self.item_sender.send(
			Item {
				typ: i,
				pos: self.start,
				val: self.input[self.start..self.pos].to_string(),
				line: self.line,
			}
		).unwrap();
		self.start = self.pos;
	}

	fn emit_with_trimmed_quotes(&mut self, i: ItemType) {
		let matcher: &[_] = &['"' as char];
		self.item_sender.send(
			Item {
				typ: i,
				pos: self.start,
				val: self.input[self.start..self.pos].trim_matches(matcher).to_string(),
				line: self.line,
			}
		).unwrap();
		self.start = self.pos;
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


	// errorf returns an error token and terminates the scan by passing
	// back a Option::None that will be the next state, closing the channel.
	fn errorf(&mut self, errstr: &str) -> Option<State> {
		self.item_sender.send(
			Item {
				typ: ItemType::Error,
				pos: self.start,
				val: errstr.to_string(),
				line: self.line,
			}
		).unwrap();
		None
	}

	// run runs the state machine for the lexer.
	pub fn run(mut self) ->Self{
		let mut sfn: StateFn = Self::lex_prop_name;

		while let Some(outer) = sfn(&mut self) {
			sfn = outer.inner;
		}
		self
		// l is dropped, l.item_sender is closed
	}

	// lexPropName scans until a colon or a semicolon
	fn lex_prop_name(l: &mut Lexer) -> Option<State> {
		l.accept_run(ALLOWED_PARAMETER_NAME_CHARS);
		if l.pos == l.start {
			return l.errorf("expected one or more alphanumerical characters or '-'");
		}
		if l.input[l.start..l.pos].to_uppercase() == COMP_BEGIN_S {
			l.emit(ItemType::Begin);
			return state!(lex_before_comp_name);
		}
		if l.input[l.start..l.pos].to_uppercase() == COMP_END_S {
			l.emit(ItemType::End);
			return state!(lex_before_comp_name);
		}

		l.emit(ItemType::Id);
		return state!(lex_before_value);
	}

	fn lex_before_comp_name(l: &mut Lexer) -> Option<State> {
		if l.accept(":") {
			l.ignore();
			return state!(lex_comp_name);
		}

		return l.errorf("expected ':'");
	}

	fn lex_comp_name(l: &mut Lexer) -> Option<State> {
		if let Rune::EOF = l.peek() {
			return l.errorf("component name can't have length 0");
		}
		l.accept_run(ALLOWED_PARAMETER_NAME_CHARS);
		match l.peek() {
			Rune::EOF => {
				l.emit(ItemType::CompName);
				None
			}
			_ => {
				l.ignore();
				return l.errorf("unexpected character, expected eol, alphanumeric or '-'");
			}
		}
	}

	fn lex_before_value(l: &mut Lexer) -> Option<State> {
		if l.accept(":") {
			l.ignore();
			return state!(lex_value);
		}
		if l.accept(";") {
			l.ignore();
			return state!(lex_param_name);
		}
		return l.errorf("expected ':' or ';'");
	}

	fn lex_param_name(l: &mut Lexer) -> Option<State> {
		l.accept_run(ALLOWED_PARAMETER_NAME_CHARS);
		if l.pos == l.start {
			return l.errorf("name must not be empty");
		}
		l.emit(ItemType::Id);
		if l.accept("=") {
			l.ignore();
			return state!(lex_param_value);
		}
		return l.errorf("expected '='");
	}

	fn lex_param_value(l: &mut Lexer) -> Option<State> {
		if l.accept("\"") {
			return state!(lex_param_q_value);
		}
		l.accept_run_unless("\",;:");
		l.emit(ItemType::ParamValue);
		return state!(lex_after_param_value);
	}
	fn lex_param_q_value(l: &mut Lexer) -> Option<State> {
		l.accept_run_unless("\"");

		if let Rune::Valid('"') = l.next(){
			l.emit_with_trimmed_quotes(ItemType::ParamValue);
			return state!(lex_after_param_value);
		}

		return l.errorf("expected '\"' or other non-control-characters");
	}

	fn lex_after_param_value(l: &mut Lexer) -> Option<State> {
		if l.accept(":") {
			l.ignore(); //l.emit(itemColon)
			return state!(lex_value);
		}
		if l.accept(";") {
			l.ignore(); //l.emit(itemSemicolon)
			return state!(lex_param_name);
		}
		if l.accept(",") {
			l.ignore(); //l.emit(itemComma)
			return state!(lex_param_value);
		}
		return l.errorf("expected ',', ':' or ';'");
	}

	fn lex_value(l: &mut Lexer) -> Option<State> {
		if let Rune::EOF = l.peek() {
			return l.errorf("property value can't have length 0");
		}
		l.accept_run_unless("");
		if let Rune::EOF = l.peek() {
			l.emit(ItemType::PropValue);
			return None;
		}
		return l.errorf("unexpected character, expected eol");
	}
}
