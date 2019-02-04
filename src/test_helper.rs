//HELPER FUNCTIONS

use crate::Component;
use std::io::Cursor;
use crate::{Parser,Property,Parameters};
use std::fmt::Debug;
use std::error::Error;

pub fn test_parse(input:&str, expected:Component){
	test_parse_bytes(input.as_bytes(), expected);
}

pub fn test_parse_bytes(input:&[u8], expected:Component){
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
	p(name,value,Parameters::new())
}

pub fn pm(vals:Vec<(&str,Vec<&str>)>)->Parameters{
	let mut out=Parameters::new();
	for (k,v) in vals{
		out.insert(k.to_string(),v.iter().map(|&s|s.to_string()).collect());
	}
	out
}

pub fn assert_comp_equal(a:&Component,b:&Component){
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

//fn test_error<'a,P:Pattern<'a>>(input:&str, error:P){
pub fn test_parse_error(input:&str, error:&str){
	let x=Cursor::new(input);
	let mut p=Parser::new(x);


	expect_err(p.next_component(),error);

	//drain the parser. This should not panic!
	for _obj in p {};
}

pub fn expect_err<R:Debug, E:Error>(res:Result<R,E>, msg:&str) {
	match &res{
		Err(e) if e.to_string().contains(msg) =>(),
		Err(e) => panic!("Didn't get the expected error, got: {:?}\n\nObject:\t{:?}", e.to_string(),e),
		Ok(c) => panic!("Expected an error, but got:{:?}",c)
	}
}
