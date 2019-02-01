use std::collections::HashMap;

use crate::Component;
use crate::Parameters;
use crate::Property;
use std::io::Cursor;
use crate::Parser;

#[test]
fn parse_successful() {
	let cases = [
		//simplest case
		("BEGIN:comp\r\nEND:Comp\r\n",
		 c2("COMP")),
		//simplest case with nested Components
		("BEGIN:comp\r\nBEGIN:inner\r\nEND:inner\r\nEND:Comp\r\n",
		 c("COMP",vec![],vec![c2("INNER")])),
		//test Property
		("BEGIN:comp\r\nFEATURE:Content:'!,;.'\r\nEND:Comp\r\n",
		 c("COMP", vec![p2("FEATURE","Content:'!,;.'")],vec![])),
		//check unfolding
		("BEGIN:comp\r\nFEATURE:Conten\r\n t:'!,;.'\r\nEND:Comp\r\n",
		 c("COMP", vec![p2("FEATURE", "Content:'!,;.'")],vec![])),
		//check parameter
		("BEGIN:comp\r\nFEATURE;LANG=en:LoremIpsum\r\nEND:Comp\r\n",
		 c("COMP",vec![p("FEATURE", "LoremIpsum",pm(vec![("LANG",vec!["en"])]))],vec![])),
		//check quoted parameter
		("BEGIN:comp\r\nFEATURE;LAng=\"e;n\":LoremIpsum\r\nEND:Comp\r\n",
		 c("COMP",vec![p("FEATURE", "LoremIpsum",pm(vec![("LANG",vec!["e;n"])]))],vec![])),
		//check RFC6868-Escaping
		("BEGIN:comp\r\nFEATURE;LANG=e^^^n:LoremIpsum\r\nEND:Comp\r\n",
		 c("COMP",vec![p("FEATURE", "LoremIpsum",pm(vec![("LANG",vec!["e^\n"])]))],vec![])),
		//check multiple Parameters with multiple values, variably encoded and folded
		("BEGIN:comp\r\nFEATURE;Par1=e^'^n,\"other^,val\";PAR2=\"\r\n display:none;\",not interesting:LoremIpsum\r\nEND:Comp\r\n",
		 c("COMP",vec![p("FEATURE", "LoremIpsum", pm(vec![
							 ("PAR1",vec!["e\"\n", "other^,val"]),
							 ("PAR2",vec!["display:none;", "not interesting"])
						 ]))],vec![])),
		//check property in nested Component
		("BEGIN:comp\r\nBEGIN:iNnErCoMp\r\nFEATURE;LAng=\"e;n\":LoremIpsum\r\nEND:InNeRcOmP\r\nEND:Comp\r\n",
		c("COMP",vec![],vec![
			c("INNERCOMP",vec![p("FEATURE", "LoremIpsum",pm(vec![("LANG",vec!["e;n"])]))],vec![])
		])),
		//check property next to nested Component
		("BEGIN:comp\r\nFEATURE;LAng=\"e;n\":LoremIpsum\r\nBEGIN:iNnErCoMp\r\nEND:InNeRcOmP\r\nFEATURE;LAng2=\"e;n\":LoremIpsum\r\nEND:Comp\r\n",
		c("COMP",vec![p("FEATURE", "LoremIpsum",pm(vec![("LANG",vec!["e;n"])]))],vec![
			c("INNERCOMP",vec![p("FEATURE", "LoremIpsum",pm(vec![("LANG2",vec!["e;n"])]))],vec![])
		])),
	];

	for (input,expect) in cases.iter(){
		let x=Cursor::new(input.as_bytes());
		let mut  p=Parser::new(x);
		let got=p.next_component().unwrap().unwrap();
		assert_comp_equal(expect, &got)
	}
}

fn c(name:&str, props:Vec<Property>, comps:Vec<Component>)->Component{
	Component{
		name: name.to_string(),
		properties: props,
		sub_components: comps,
	}
}
fn c2(name:&str)->Component{
	c(name,vec![],vec![])
}

fn p(name:&str, value:&str, param:Parameters) ->Property{
	Property{
		name: name.to_string(),
		value: value.to_string(),
		parameters: param,
		old_line: None
	}
}

fn p2(name:&str, value:&str) ->Property{
	p(name,value,HashMap::new())
}

fn pm(vals:Vec<(&str,Vec<&str>)>)->Parameters{
	let mut out=Parameters::new();
	for (k,v) in vals{
		out.insert(k.to_string(),v.iter().map(|&s|s.to_string()).collect());
	}
	out
}

fn assert_comp_equal(a:&Component,b:&Component){
	assert_eq!(a.name, b.name)

}
