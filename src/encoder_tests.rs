use crate::test_helper::{c2, p2};

#[test]
fn simple() {
	let c = c2("House");
	let expected = "BEGIN:HOUSE\r\nEND:HOUSE\r\n";
	assert_eq!(c.encode_to_string().as_str(), expected)
}

#[test]
fn nested() {
	let mut c = c2("House");
	c.add_sub_component(c2("Flat"));
	let expected = "BEGIN:HOUSE\r\nBEGIN:FLAT\r\nEND:FLAT\r\nEND:HOUSE\r\n";
	assert_eq!(c.encode_to_string().as_str(), expected)
}

#[test]
fn property() {
	let mut c = c2("House");
	c.add_property(p2("Heating", "gas"));
	let expected = "BEGIN:HOUSE\r\nHEATING:gas\r\nEND:HOUSE\r\n";
	assert_eq!(c.encode_to_string().as_str(), expected)
}

#[test]
fn property_complex() {
	let mut c = c2("House");
	let mut p = p2("Heating", "electric");
	p.add_param("vendor".to_string(), "YourGas Co\"".to_string());
	p.add_param("vendor".to_string(), "City:Energy LLC".to_string());
	p.add_param("comment".to_string(), "This is a very long comment,more than 2^3 monkeys hat to sit 20 hours to write this \n thing with linebreaks.".to_string());
	c.add_property(p);
	let expected = "BEGIN:HOUSE\r\n".to_string() +
			"HEATING;" +
			"VENDOR=YourGas Co^',\"City:Energy LLC\";" +
			"COMMENT=\"This is a very long \r\n comment,more than 2^^3 monkeys hat to sit 20 hours to write this ^n thing \r\n with linebreaks.\":" +
			"electric\r\n" +
			"END:HOUSE\r\n";
	let alternative = "BEGIN:HOUSE\r\n".to_string() +
			"HEATING;" +
			"COMMENT=\"This is a very long comment,more than 2^^3 monkeys hat to \r\n sit 20 hours to write this ^n thing with linebreaks.\";" +
			"VENDOR=YourGas Co^',\r\n \"City:Energy LLC\":" +
			"electric\r\n" +
			"END:HOUSE\r\n";
	let s=c.encode_to_string();
	assert!(s == expected || s == alternative, "\nGot:     {:?}\nExpected:{:?}\nOr      :{:?}",s,expected,alternative);
}

#[test]
fn utf8_folding() {
	let mut c = c2("House");
	let mut p = p2("Heating", "electric");
	p.add_param("comment".to_string(), "This is a very long comment,11 monkeys hat to paint 200 \u{2764}s to write this thing.".to_string());
	c.add_property(p);
	let expected = "BEGIN:HOUSE\r\n".to_string() +
			"HEATING;" +
			"COMMENT=\"This is a very long comment,11 monkeys hat to paint 200 \r\n \u{2764}s to write this thing.\":" +
			"electric\r\n" +
			"END:HOUSE\r\n";
	assert_eq!(c.encode_to_string(), expected);
}

#[test]
fn nested_complex() {
	let mut c = c2("House");
	let mut p = p2("Heating", "electric");
	p.add_param("vendor".to_string(), "YourGas Co\"".to_string());
	p.add_param("vendor".to_string(), "City:Energy LLC".to_string());
	p.add_param("comment".to_string(), "This is a very long comment,more than 2^3 monkeys hat to sit 20 hours to write this \n thing with linebreaks.".to_string());
	c.add_property(p);
	let mut p = p2("Heating2", "electric2");
	p.add_param("vendor".to_string(), "YourGas Co\"".to_string());
	p.add_param("vendor".to_string(), "City:Energy LLC".to_string());
	p.add_param("comment".to_string(), "This is a very long comment,more than 2^3 monkeys hat to sit 20 hours to write this \n thing with linebreaks.".to_string());
	let mut c2=c2("Flat");
	c2.add_property(p);
	c.add_sub_component(c2);
	let expected="BEGIN:HOUSE\r\n".to_string() +
			"HEATING;"+
			"VENDOR=YourGas Co^',\"City:Energy LLC\";"+
			"COMMENT=\"This is a very long \r\n comment,more than 2^^3 monkeys hat to sit 20 hours to write this ^n thing \r\n with linebreaks.\":electric\r\n"+
			"BEGIN:FLAT\r\n"+
			"HEATING2;"+
			"VENDOR=YourGas Co^',\"City:Energy LLC\";"+
			"COMMENT=\"This is a very long\r\n  comment,more than 2^^3 monkeys hat to sit 20 hours to write this ^n thing\r\n  with linebreaks.\":electric2\r\n"+
			"END:FLAT\r\nEND:HOUSE\r\n";
	let alt1="BEGIN:HOUSE\r\n".to_string() +
			"HEATING;"+
			"COMMENT=\"This is a very long comment,more than 2^^3 monkeys hat to \r\n sit 20 hours to write this ^n thing with linebreaks.\";" +
			"VENDOR=YourGas Co^',\r\n \"City:Energy LLC\":electric\r\n" +
			"BEGIN:FLAT\r\n"+
			"HEATING2;"+
			"VENDOR=YourGas Co^',\"City:Energy LLC\";"+
			"COMMENT=\"This is a very long\r\n  comment,more than 2^^3 monkeys hat to sit 20 hours to write this ^n thing\r\n  with linebreaks.\":electric2\r\n"+
			"END:FLAT\r\nEND:HOUSE\r\n";
	let alt2="BEGIN:HOUSE\r\n".to_string() +
			"HEATING;"+
			"VENDOR=YourGas Co^',\"City:Energy LLC\";"+
			"COMMENT=\"This is a very long \r\n comment,more than 2^^3 monkeys hat to sit 20 hours to write this ^n thing \r\n with linebreaks.\":electric\r\n"+
			"BEGIN:FLAT\r\n"+
			"HEATING2;"+
			"COMMENT=\"This is a very long comment,more than 2^^3 monkeys hat to\r\n  sit 20 hours to write this ^n thing with linebreaks.\";" +
			"VENDOR=YourGas Co^'\r\n ,\"City:Energy LLC\":electric2\r\n" +
			"END:FLAT\r\nEND:HOUSE\r\n";
	let alt3="BEGIN:HOUSE\r\n".to_string() +
			"HEATING;"+
			"COMMENT=\"This is a very long comment,more than 2^^3 monkeys hat to \r\n sit 20 hours to write this ^n thing with linebreaks.\";" +
			"VENDOR=YourGas Co^',\r\n \"City:Energy LLC\":electric\r\n" +
			"BEGIN:FLAT\r\n"+
			"HEATING2;"+
			"COMMENT=\"This is a very long comment,more than 2^^3 monkeys hat to\r\n  sit 20 hours to write this ^n thing with linebreaks.\";" +
			"VENDOR=YourGas Co^'\r\n ,\"City:Energy LLC\":electric2\r\n" +
			"END:FLAT\r\nEND:HOUSE\r\n";

	let s=c.encode_to_string();
	assert!(s == expected || s == alt1 || s == alt2 || s == alt3,
			"\nGot:     {:?}\nExpected:{:?}\nOr      :{:?}\nOr      :{:?}\nOr      :{:?}",s,expected,alt1,alt2,alt3);
}
