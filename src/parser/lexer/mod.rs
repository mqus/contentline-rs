use std::error::Error;
use std::fmt;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;

pub use crate::parser::lexer::internals::ALLOWED_PARAMETER_NAME_CHARS;
use crate::parser::lexer::internals::Lexer;

pub type Pos = usize;

mod internals;

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


// lex creates a new scanner for the input string.
pub fn new(line: u32, input: String) -> LexerHandle {
	let (s, r) = mpsc::channel();

	let l = Lexer::new(line, input.clone(), s);

	let jh = thread::spawn(move || l.run());

	let lh = LexerHandle {
		item_receiver: r,
		join_handle: jh,
		line: (input,line),
	};

	lh
}

pub struct LexerHandle {
	item_receiver: Receiver<Item>,
	join_handle: JoinHandle<Lexer>,
	line: (String,u32),
}

impl LexerHandle {
	pub fn next_item(&self) -> Option<Item> {
		self.item_receiver.recv().ok()
	}

	pub fn get_line(&self) ->(String,u32){
		return (self.line.0.clone(),self.line.1)
	}

	pub fn drain(&self) {
		for _item in self.item_receiver.iter() {}
	}

	pub fn drain_and_join(self) -> Lexer {
		self.drain();
		self.join_handle.join().unwrap_or_else(|_| panic!("Couldn't join, Thread panicked!"))
	}
}

//Can't simply implement drop, because parser::get_next_item moves input out before dropping the Handle
/*impl Drop for LexerHandle {
	fn drop(&mut self) {
		self.drain()
		//MAYBE clean up thread
		//self.join_handle.join()
	}
}*/


#[derive(Debug)]
pub struct LexerError {
	ctx: String,
	msg: String,
	i: Item,
	line: u32,
}

const ERROR_CONTEXT_RADIUS: usize = 20;

impl LexerError {
	pub fn new(ctx: LexerHandle, i: Item, msg: &str, line: u32) -> Self {
		LexerError {
			ctx: ctx.drain_and_join().input,
			msg: msg.to_string(),
			i,
			line,
		}
	}
}

impl Error for LexerError {}

impl fmt::Display for LexerError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut msg = self.msg.as_str();
		let ctx = self.ctx.as_str();

		let pos1 = self.i.pos;
		let mut pos2 = self.i.pos + self.i.val.len();

		if msg == "" {
			msg = self.i.val.as_str();
			pos2 = pos1 + 1;
		}

		let prefix=if pos1 > ERROR_CONTEXT_RADIUS {
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

		return match suffix.len() {
			0 => writeln!(f, "{}: \t{}: {}<HERE>", self.line, msg, &prefix),
			1 => writeln!(f, "{}: \t{}: {} >{}<", self.line, msg, &prefix, &suffix[..pos2 - pos1]),
			_ => writeln!(f, "{}: \t{}: {} >{}< {}", self.line, msg, &prefix, &suffix[..pos2 - pos1], &suffix[pos2 - pos1..]),
		};
	}
}
