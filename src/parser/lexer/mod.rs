use std::fmt;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;
use std::error::Error;
use crate::parser::lexer::internals::Lexer;

pub type Pos = usize;

mod internals;

#[derive(Clone,Debug)]
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
#[derive(Clone,Debug)]
pub enum ItemType {
	// error occurred; value is text of error
	Error,
	// equals ('=') used for character-assignment
	//Equals,
	// :
	//Colon,
	// ;
	//Semicolon,
	// ,
	//Comma,
	// the value of a property parameter, can contain ^^, ^' or ^n
	ParamValue,
	// the value of a property, if the property is of type TEXT, the value can contain \\ , \; , \, , \n or \N
	PropValue,
	// the Property Name
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

	let l = Lexer::new(line,input.clone(),s);

	let jh = thread::spawn(move || l.run());

	let lh = LexerHandle {
		item_receiver: r,
		join_handle: jh,
		input,
	};

	lh
}

pub struct LexerHandle {
	item_receiver: Receiver<Item>,
	join_handle: JoinHandle<Lexer>,
	pub input:String,
}


impl LexerHandle {
	pub fn next_item(&self) -> Option<Item> {
		self.item_receiver.recv().ok()
	}

	pub fn drain(&self) {
		for s in self.item_receiver.iter() {
		}
	}

	pub fn drain_and_join(mut self)->Lexer{
		self.drain();
		self.join_handle.join().unwrap_or_else(|_|panic!("Couldn't join, Thread panicked!"))
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
}

const ERROR_CONTEXT_RADIUS: usize = 20;

impl LexerError {
	pub fn new(line: String, i: Item, msg: &str) -> Self {
		LexerError {
			ctx: line,
			msg: msg.to_string(),
			i,
		}
	}
}

impl Error for LexerError {}

impl fmt::Display for LexerError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut msg = self.msg.as_str();
		let ctx = self.ctx.as_str();

		let (mut prefix, mut suffix) = ("".to_owned(), "".to_owned());

		let pos1 = self.i.pos;
		let mut pos2 = self.i.pos + self.i.val.len();

		if msg == "" {
			msg = self.i.val.as_str();
			pos2 = pos1 + 1;
		}

		if pos1 > ERROR_CONTEXT_RADIUS {
			prefix = "...".to_owned() + &ctx[pos1 - ERROR_CONTEXT_RADIUS..pos1];
		} else {
			prefix = ctx[..pos1].to_owned();
		}

		if ctx.len() > ERROR_CONTEXT_RADIUS + pos2 {
			suffix = ctx[pos1..pos2 + ERROR_CONTEXT_RADIUS].to_owned() + "...";
		} else if pos1 < ctx.len() {
			suffix = ctx[pos1..].to_owned();
		}
		return match suffix.len() {
			0 => writeln!(f, "{}: \t{}<HERE>", msg, &prefix),
			1 => writeln!(f, "{}: \t{} >{}<", msg, &prefix, &suffix[..pos2 - pos1]),
			_ => writeln!(f, "{}: \t{} >{}< {}", msg, &prefix, &suffix[..pos2 - pos1], &suffix[pos2 - pos1..]),
		};
	}
}
