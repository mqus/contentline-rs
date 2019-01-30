use crate::lexer::LexerHandle;
use std::error::Error;
use crate::lexer;
use crate::lexer::Item;
use core::fmt;
use crate::lexer::ItemType;

struct Parser{
	lexer:Option<LexerHandle>,
	line:u32,
	r:u32,//TODO:Reader
}

impl Parser{
	//getNextItem returns the next lexer item, feeding (unfolded) lines into the lexer if neccessary.
	// It also converts identifiers (itemCompName, itemID) into upper case, errors encountered by the
	// lexer into 'error' values and property parameter values into their original value (without escaped characters).
	fn get_next_item(&mut self) -> Result<Item,Box<dyn Error>>{
		if let None=self.lexer{
			let line=self.read_unfolded_line()?;
			self.lexer=Some(lexer::new(self.line,line));
		}
		//TODO handle second unwrap (lexer eof)
		let mut i=self.lexer.get_or_insert(unreachable!()).next_item().unwrap();

		match i.typ{
			ItemType::Error =>{
				let e=Box::new(
					ParserError::new(self.lexer.get_or_insert(unreachable!()).input,i,""));
				self.lexer=None;
				return Err(e);
			},
			ItemType::CompName =>{
				i.val=i.val.to_uppercase();
				self.lexer=None;
			},
			ItemType::Id => i.val=i.val.to_uppercase(),
			ItemType::PropValue => self.lexer=None,
			ItemType::ParamValue => i.val=unescape_param_value(i.val),
			_=>{}
		}
		Ok(i)
	}

	fn read_unfolded_line(&mut self) ->Result<String,Box<dyn Error>>{
		unimplemented!()

		//TODO
	}
}

fn unescape_param_value(escaped:String)->String{
	//TODO
	escaped
}

#[derive(Debug)]
struct ParserError{
	ctx:String,
	msg:String,
	i:Item,
}
const ERROR_CONTEXT_RADIUS: usize = 20;
impl ParserError{
	fn new(line:String, i: Item, msg: &str) -> Self {
		ParserError {
			ctx: line,
			msg:msg.to_string(),
			i,
		}
	}
}
impl Error for ParserError{}
impl fmt::Display for ParserError{

	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut msg=self.msg.as_str();
		let ctx=self.ctx.as_str();

		let (mut prefix,mut suffix) = ("".to_owned(),"".to_owned());

		let pos1=self.i.pos;
		let mut pos2=self.i.pos+self.i.val.len();

		if msg == ""{
			msg=self.i.val.as_str();
			pos2=pos1+1;
		}

		if pos1 > ERROR_CONTEXT_RADIUS{
			prefix = "...".to_owned() + &ctx[pos1-ERROR_CONTEXT_RADIUS..pos1];
		} else{
			prefix = ctx[..pos1].to_owned();
		}

		if ctx.len() > ERROR_CONTEXT_RADIUS + pos2 {
			suffix = ctx[pos1..pos2+ERROR_CONTEXT_RADIUS].to_owned() + "...";
		} else if pos1 < ctx.len() {
			suffix = ctx[pos1..].to_owned();
		}
		return match suffix.len(){
			0 => writeln!(f,"{}: \t{}<HERE>"  , msg, &prefix),
			1 => writeln!(f,"{}: \t{} >{}<"   , msg, &prefix, &suffix[..pos2-pos1]),
			_ => writeln!(f,"{}: \t{} >{}< {}", msg, &prefix, &suffix[..pos2-pos1], &suffix[pos2-pos1..]),
		}
	}
}
