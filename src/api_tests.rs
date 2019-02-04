use std::io::Cursor;
use crate::test_helper::expect_err;

#[test]
fn from_unbuf(){
	let mut p=crate::Parser::from_unbuffered(Cursor::new(vec![]));
	if let Ok(None) = p.next_component(){

	}else{
		panic!("didn't return Ok(None) from empty unbuffered input")
	}
}

#[test]
fn new_prop_empty(){
	crate::Property::new_empty("Name".to_string(), "".to_string()).unwrap();
}

#[test]
fn new_prop_empty_wrong(){
	let p=crate::Property::new_empty("Nam e".to_string(), "".to_string());
	expect_err(p,"property name \"Nam e\" is invalid: character ' ' is not allowed");

}

#[test]
fn new_comp_empty(){
	crate::Component::new_empty("Name".to_string()).unwrap();
}

#[test]
fn new_comp_empty_wrong(){
	let c=crate::Component::new_empty("Nam e".to_string());
	expect_err(c,"component name \"Nam e\" is invalid: character ' ' is not allowed");
}

#[test]
fn new_comp_full(){
	let mut p=crate::Property::new_empty("Name".to_string(), "".to_string()).unwrap();
	p.add_param("hi".to_string(),"lo".to_string()).unwrap();
	let mut c1=crate::Component::new_empty("Name".to_string()).unwrap();
	c1.add_property(p);
	crate::Component::new("Name".to_string(),vec![],vec![c1]).unwrap();

}



