use std::collections::HashMap;

use crate::Component;
use crate::Parameters;
use crate::Property;
use std::io::Cursor;
use crate::Parser;

//HELPER FUNCTIONS

fn test(input:&str, expected:Component){
	test_bytes(input.as_bytes(), expected);
}

fn test_bytes(input:&[u8], expected:Component){
	let x=Cursor::new(input);
	let mut p=Parser::new(x);
	let got=p.next_component().unwrap().unwrap();
	assert_comp_equal(&got,&expected);
	if let Some(x) = p.next_component().unwrap(){
		panic!("expected EOF, but got:\n{:#?}",x)
	}
}

//Convenience functions for quickly setting up Components, Properties and Parameters
pub fn c(name:&str, props:Vec<Property>, comps:Vec<Component>)->Component{
	Component{
		name: name.to_string(),
		properties: props,
		sub_components: comps,
	}
}
pub fn c2(name:&str)->Component{
	c(name,vec![],vec![])
}

pub fn p(name:&str, value:&str, param:Parameters) ->Property{
	Property{
		name: name.to_string(),
		value: value.to_string(),
		parameters: param,
		old_line: None
	}
}

pub fn p2(name:&str, value:&str) ->Property{
	p(name,value,HashMap::new())
}

pub fn pm(vals:Vec<(&str,Vec<&str>)>)->Parameters{
	let mut out=Parameters::new();
	for (k,v) in vals{
		out.insert(k.to_string(),v.iter().map(|&s|s.to_string()).collect());
	}
	out
}

fn assert_comp_equal(a:&Component,b:&Component){
	assert_eq!(a.name, b.name, "component names");
	assert_eq!(a.properties.len(),b.properties.len(), "property count, components:\n{:#?}\n{:#?}",a,b);
	assert_eq!(a.sub_components.len(),b.sub_components.len(), "subcomponent count");

	for i in 0..a.properties.len(){
		assert_prop_equal(&a.properties[i], &b.properties[i]);
	}

	for i in 0..a.sub_components.len(){
		assert_comp_equal(&a.sub_components[i], &b.sub_components[i]);
	}
}

fn assert_prop_equal(a:&Property,b:&Property){
	assert_eq!(a.name,b.name,"property names");
	assert_eq!(a.value,b.value,"property values");
	assert_eq!(a.parameters.len(),b.parameters.len(),"parameter counts");

	for (a_p,a_values) in &a.parameters{
		for a_val in a_values{
			assert!(b.parameters.get(a_p).unwrap().contains(a_val))
		}
	}
	//this tested if all values of a are in b. Now we test the reverse
	for (b_p,b_values) in &b.parameters{
		for b_val in b_values{
			assert!(b.parameters.get(b_p).unwrap().contains(b_val))
		}
	}

}

//TESTS: Successful parses



#[test]
fn parse_empty(){
	let x=Cursor::new("".as_bytes());
	let mut p=Parser::new(x);
	let got=p.next_component().unwrap();
	assert!(got.is_none());
}

#[test]
fn parse_simple(){
	test("BEGIN:comp\r\nEND:Comp\r\n", c2("COMP"))
}

#[test]
fn parse_simple_nested(){
	test("BEGIN:comp\r\nBEGIN:inner\r\nEND:inner\r\nEND:Comp\r\n",
		 c("COMP",vec![],vec![c2("INNER")]))
}

#[test]
fn parse_with_property(){
	test("BEGIN:comp\r\nFEATURE:Content:'!,;.'\r\nEND:Comp\r\n",
		 c("COMP", vec![p2("FEATURE","Content:'!,;.'")],vec![]))
}

#[test]
fn parse_unfolding(){
	test("BEGIN:comp\r\nFEATURE:Conten\r\n t:'!,;.'\r\nEND:Comp\r\n",
		 c("COMP", vec![p2("FEATURE", "Content:'!,;.'")],vec![]))
}

#[test]
fn parse_parameter(){
	test("BEGIN:comp\r\nFEATURE;LANG=en:LoremIpsum\r\nEND:Comp\r\n",
		 c("COMP",vec![p("FEATURE", "LoremIpsum",pm(vec![("LANG",vec!["en"])]))],vec![]))
}

#[test]
fn parse_quoted_parameter(){
	test("BEGIN:comp\r\nFEATURE;LAng=\"e;n\":LoremIpsum\r\nEND:Comp\r\n",
		 c("COMP",vec![p("FEATURE", "LoremIpsum",pm(vec![("LANG",vec!["e;n"])]))],vec![]))
}

#[test]
fn parse_with_rfc6868_escaping(){
	test("BEGIN:comp\r\nFEATURE;LANG=e^^^n:LoremIpsum\r\nEND:Comp\r\n",
		 c("COMP",vec![p("FEATURE", "LoremIpsum",pm(vec![("LANG",vec!["e^\n"])]))],vec![]))
}

#[test]
fn parse_complex(){
	test("BEGIN:comp\r\nFEATURE;Par1=e^'^n,\"other^,val\";PAR2=\"\r\n display:none;\",not interesting:LoremIpsum\r\nEND:Comp\r\n",
		 c("COMP",vec![p("FEATURE", "LoremIpsum", pm(vec![
			 ("PAR1",vec!["e\"\n", "other^,val"]),
			 ("PAR2",vec!["display:none;", "not interesting"])
		 ]))],vec![]))
}

#[test]
fn parse_nested_component(){
	test("BEGIN:comp\r\nBEGIN:iNnErCoMp\r\nFEATURE;LAng=\"e;n\":LoremIpsum\r\nEND:InNeRcOmP\r\nEND:Comp\r\n",
		 c("COMP",vec![],vec![
			 c("INNERCOMP",vec![p("FEATURE", "LoremIpsum",pm(vec![("LANG",vec!["e;n"])]))],vec![])
		 ]))
}

#[test]
fn parse_nested_and_property(){
	test("BEGIN:comp\r\nFEATURE;LAng=\"e;n\":LoremIpsum\r\nBEGIN:iNnErCoMp\r\nEND:InNeRcOmP\r\nFEATURE;LAng2=\"e;n\":LoremIpsum\r\nEND:Comp\r\n",
		 c("COMP",vec![
			 p("FEATURE", "LoremIpsum",pm(vec![("LANG",vec!["e;n"])])),
			 p("FEATURE", "LoremIpsum",pm(vec![("LANG2",vec!["e;n"])]))
		 ],vec![c("INNERCOMP",vec![],vec![])]))
}

#[test]
fn parse_two_components(){
	let input="BEGIN:comp\r\nEND:Comp\r\nBEGIN:porp\r\nEND:poRp\r\n";
	let expect1=c2("COMP");
	let expect2=c2("PORP");

	let x=Cursor::new(input.as_bytes());
	let mut  p=Parser::new(x);
	let got1=p.next_component().unwrap().unwrap();
	let got2=p.next_component().unwrap().unwrap();
	assert_comp_equal(&got1,&expect1);
	assert_comp_equal(&got2,&expect2);
	if let Some(x) = p.next_component().unwrap(){
		panic!("expected EOF, but got:\n{:#?}",x)
	}
}

#[test]
fn parse_utf_splitting_fold(){
	//test if the parser allows for folds (a \r\n newline followed by a space) within a utf8-codepoint sequence.

	let expected=c("COMP", vec![p2("FEATURE", "\u{2764}Content:'!,;.'")],vec![]);

	let prefix="BEGIN:comp\r\nFEATURE:";
	let bytes=vec![0xE2_u8,0x9D,0xA4];//UTF8 Heart Character
	let fold="\r\n ";
	let suffix="Content:'!,;.'\r\nEND:Comp\r\n";

	let mut x=vec![];

	x.extend_from_slice(prefix.as_bytes());
	x.extend_from_slice(&bytes[0..2]);
	x.extend_from_slice(fold.as_bytes());
	x.extend_from_slice(&bytes[2..3]);
	x.extend_from_slice(suffix.as_bytes());
	test_bytes(x.as_slice(),expected);
}

#[test]
fn parse_cornercase_fold(){
	let input="begin:comp\r\n \r\n \r\nend:comp\r\n";
	let expected=c2("COMP");
	test(input,expected)

}


//fn test_error<'a,P:Pattern<'a>>(input:&str, error:P){
fn test_error(input:&str, error:&str){
	let x=Cursor::new(input);
	let mut p=Parser::new(x);

	match p.next_component(){
		Err(e) => {
			//check if error is the expected error
			if !e.to_string().contains(error){
				panic!("Didn't get the expected error, got:\n{:?}\nDescription: {:?}",e, e.to_string());
			}
		},
		Ok(c) => panic!("Expected an error, but got:{:?}",c),
	};


}


//TESTS: EXPECTED ERRORS /TODO
#[test]
fn empty_line(){
	test_error("\r\n","expected one or more alphanumerical characters or '-'");
}

#[test]
fn wrong_linebreak(){
	test_error("\n","expected one or more alphanumerical characters or '-'");
}

#[test]
fn wrong_comp_begin(){
	test_error("BEG\r\n","unexpected identifier, expected BEGIN");
}

#[test]
fn wrong_comp_begin2(){
	test_error("BEG:\r\n","unexpected identifier, expected BEGIN");
}

#[test]
fn wrong_comp_begin3(){
	test_error("BEGIN\r\n","1: \texpected ':': BEGIN<HERE>\n");
}

//wrong prop id

//wrong param id

//wrong newlines

//
