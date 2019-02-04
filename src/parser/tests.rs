use std::io::Cursor;
use crate::Parser;
use crate::test_helper::c2;
use crate::test_helper::c;
use crate::test_helper::p2;
use crate::test_helper::p;
use crate::test_helper::pm;
use crate::test_helper::test_parse;
use crate::test_helper::assert_comp_equal;
use crate::test_helper::test_parse_bytes;
use crate::test_helper::test_parse_error;

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
	test_parse("BEGIN:comp\r\nEND:Comp\r\n", c2("COMP"))
}

#[test]
fn parse_simple_nested(){
	test_parse("BEGIN:comp\r\nBEGIN:inner\r\nEND:inner\r\nEND:Comp\r\n",
		 c("COMP",vec![],vec![c2("INNER")]))
}

#[test]
fn parse_with_property(){
	test_parse("BEGIN:comp\r\nFEATURE:Content:'!,;.'\r\nEND:Comp\r\n",
		 c("COMP", vec![p2("FEATURE","Content:'!,;.'")],vec![]))
}

#[test]
fn parse_unfolding(){
	test_parse("BEGIN:comp\r\nFEATURE:Conten\r\n t:'!,;.'\r\nEND:Comp\r\n",
		 c("COMP", vec![p2("FEATURE", "Content:'!,;.'")],vec![]))
}

#[test]
fn parse_parameter(){
	test_parse("BEGIN:comp\r\nFEATURE;LANG=en:LoremIpsum\r\nEND:Comp\r\n",
		 c("COMP",vec![p("FEATURE", "LoremIpsum",pm(vec![("LANG",vec!["en"])]))],vec![]))
}

#[test]
fn parse_quoted_parameter(){
	test_parse("BEGIN:comp\r\nFEATURE;LAng=\"e;n\":LoremIpsum\r\nEND:Comp\r\n",
		 c("COMP",vec![p("FEATURE", "LoremIpsum",pm(vec![("LANG",vec!["e;n"])]))],vec![]))
}

#[test]
fn parse_with_rfc6868_escaping(){
	test_parse("BEGIN:comp\r\nFEATURE;LANG=e^^^n:LoremIpsum\r\nEND:Comp\r\n",
		 c("COMP",vec![p("FEATURE", "LoremIpsum",pm(vec![("LANG",vec!["e^\n"])]))],vec![]))
}

#[test]
fn parse_complex(){
	test_parse("BEGIN:comp\r\nFEATURE;Par1=e^'^n,\"other^,val\";PAR2=\"\r\n display:none;\",not interesting:LoremIpsum\r\nEND:Comp\r\n",
		 c("COMP",vec![p("FEATURE", "LoremIpsum", pm(vec![
			 ("PAR1",vec!["e\"\n", "other^,val"]),
			 ("PAR2",vec!["display:none;", "not interesting"])
		 ]))],vec![]))
}

#[test]
fn parse_nested_component(){
	test_parse("BEGIN:comp\r\nBEGIN:iNnErCoMp\r\nFEATURE;LAng=\"e;n\":LoremIpsum\r\nEND:InNeRcOmP\r\nEND:Comp\r\n",
		 c("COMP",vec![],vec![
			 c("INNERCOMP",vec![p("FEATURE", "LoremIpsum",pm(vec![("LANG",vec!["e;n"])]))],vec![])
		 ]))
}

#[test]
fn parse_nested_and_property(){
	test_parse("BEGIN:comp\r\nFEATURE;LAng=\"e;n\":LoremIpsum\r\nBEGIN:iNnErCoMp\r\nEND:InNeRcOmP\r\nFEATURE;LAng2=\"e;n\":LoremIpsum\r\nEND:Comp\r\n",
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
	test_parse_bytes(x.as_slice(),expected);
}

#[test]
fn parse_cornercase_fold(){
	let input="begin:comp\r\n \r\n \r\nend:comp\r\n";
	let expected=c2("COMP");
	test_parse(input,expected)

}

#[test]
fn parse_no_component1(){
	let input="";
	let x=Cursor::new(input.as_bytes());
	let mut p=Parser::new(x);
	if let Some(x) = p.next_component().unwrap(){
		panic!("expected EOF, but got:\n{:#?}",x)
	}
}

#[test]//This shouldn't really work but is an unavoidable quirk because "<...xyz>\n" is indistinguishable from "<...xyz>" (without \n)
fn parse_no_component2(){
	let input="\n";
	let x=Cursor::new(input.as_bytes());
	let mut p=Parser::new(x);
	if let Some(x) = p.next_component().unwrap(){
		panic!("expected EOF, but got:\n{:#?}",x)
	}
}


//TESTS: EXPECTED ERRORS /TODO
#[test]
fn empty_line(){
	test_parse_error("\r\n", "expected one or more alphanumerical characters or '-'");
}

#[test]
fn wrong_linebreak(){
	test_parse_error("\n\n", "line 1: expected CR ('\\r') before LF in empty line");
}

#[test]
fn wrong_line_begin(){
	test_parse_error(":\r\n", "line 1: \texpected one or more alphanumerical characters or '-':  >:<\n");
}

#[test]
fn wrong_comp_begin(){
	test_parse_error("BEG\r\n", "line 1: \texpected BEGIN:  >BEG< \n");
}

#[test]
fn wrong_comp_begin2(){
	test_parse_error("BEG:\r\n", "line 1: \texpected BEGIN:  >BEG< :\n");
}

#[test]
fn wrong_comp_begin3(){
	test_parse_error("BEGIN\r\n", "1: \texpected ':': BEGIN<HERE>\n");
}

#[test]
fn wrong_comp_begin4(){
	test_parse_error("BEGIN:\r\n", "line 1: \tcomponent name mustn't have length 0: BEGIN:<HERE>\n");
}

#[test]
fn wrong_comp_begin5(){
	test_parse_error("BEGIN:HI\n", "line 1: \texpected CR ('\\r') before LF: BEGIN:HI<HERE>\n");
}

#[test]
fn wrong_comp_name(){
	test_parse_error("BEGIN:HI\u{2764}\r\n", "line 1: \tunexpected character, expected eol, alphanumeric or '-': BEGIN:HI >❤< \n");
}

//wrong prop id
#[test]
fn wrong_prop_begin1(){
	test_parse_error("BEGIN:co\r\n:\r\n", "line 2: \texpected one or more alphanumerical characters or '-':  >:<\n");
}

#[test]
fn wrong_prop_begin2(){
	test_parse_error("BEGIN:co\r\nw :\r\n", "line 2: \texpected ':' or ';': w > < :\n");
}

#[test]
fn wrong_prop_begin3(){
	test_parse_error("BEGIN:co\r\nwas:\r\n", "line 2: \tproperty value can\'t have length 0: was:<HERE>\n");
}

#[test]
fn wrong_prop_param(){
	test_parse_error("BEGIN:co\r\nwas;\r\n", "line 2: \tparameter name must not be empty: was;<HERE>\n");
}

#[test]
fn wrong_prop_param2(){
	test_parse_error("BEGIN:co\r\nwas;x\r\n", "line 2: \texpected \'=\': was;x<HERE>\n");
}

#[test]
fn wrong_prop_param3(){
	test_parse_error("BEGIN:co\r\nwas; x\r\n", "line 2: \tparameter name must not be empty: was; > < x\n");
}

#[test]
fn wrong_prop_param4(){
	test_parse_error("BEGIN:co\r\nwas;x =\r\n", "line 2: \texpected '=': was;x > < =\n");
}

#[test]
fn wrong_prop_param5(){
	test_parse_error("BEGIN:co\r\nwas;=x\r\n", "line 2: \tparameter name must not be empty: was; >=< x\n");
}

#[test]
fn wrong_comp_end1(){
	test_parse_error("BEGIN:co\r\nwas:x\r\n", "line 3: Unexpected end of file or stream, expected END:CO");
}

#[test]
fn wrong_comp_end2(){
	test_parse_error("BEGIN:co\r\nwas:x\r\nend:x\r\n", "line 3: \texpected \"END:CO\": end: >x<\n");
}


#[test]
fn utf8_1(){
	let st="BEGIN:co\r\nwas;=x";
	let mut data=vec![];
	data.extend_from_slice(st.as_bytes());
	data.push(0x82);
	data.push(b'\r');
	data.push(b'\n');
	let p = Parser::new(Cursor::new(data.as_slice()));
	for obj in p{
		match obj{
			Err(e) => assert_eq!(e.to_string(),"invalid utf-8 sequence of 1 bytes from index 6"),
			Ok(_) => {},
		}
	};
}

//wrong param id

//an example which was panicking on a fuzz
#[test]
fn fuzzing_example_1(){
	let data=vec![0x0a,0xec,0x0a,0x0d];
	let p = Parser::new(Cursor::new(data.as_slice()));
	for _obj in p{};
}

