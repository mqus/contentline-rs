pub fn unescape_param_value(escaped: &str) -> String {

	//separate ^^ from other following characters to avoid wrong decoding of eg "^^n"
	escaped.replace("^^", "^^ ")

			//decode newline characters
			//MAYBE change to \r\n or \r if neccessary. RFC states: dependent on the platform being used
			.replace("^n", "\n")
			.replace("^N", "\n")
			//decode " character
			.replace("^'", "\"")
			//decode separated ^ character
			.replace("^^ ", "^")
}

pub fn escape_param_value(unescaped: &str) -> String {

	//first, decode ^ character
	unescaped.replace("^", "^^")

			//decode all other characters
			.replace("\r\n", "^n")
			.replace("\r", "^n")
			.replace("\n", "^n")
			.replace("\"", "^'")
}

#[cfg(test)]
mod tests {
	use crate::parser::rfc6868::*;

	#[test]
	fn encode() {
		assert_eq!(escape_param_value(""), "");
		assert_eq!(escape_param_value("^"), "^^");
		assert_eq!(escape_param_value("\""), "^'");
		assert_eq!(escape_param_value("\n"), "^n");
		assert_eq!(escape_param_value("\r\n"), "^n");
		assert_eq!(escape_param_value("\r\n\r\n"), "^n^n");
		assert_eq!(escape_param_value("\r\r\n"), "^n^n");
		assert_eq!(escape_param_value("\r\n\n"), "^n^n");
		assert_eq!(escape_param_value("^m"), "^^m");
		assert_eq!(escape_param_value("^n"), "^^n");
		assert_eq!(escape_param_value("^\""), "^^^'");
		assert_eq!(escape_param_value("^\"^\n\"^N^"), "^^^'^^^n^'^^N^^");
		assert_eq!(escape_param_value("^^"), "^^^^");
		assert_eq!(escape_param_value("^^n"), "^^^^n");
		assert_eq!(escape_param_value("^^\n"), "^^^^^n");
	}

	#[test]
	fn decode() {
		assert_eq!(unescape_param_value(""), "");
		assert_eq!(unescape_param_value("^^"), "^");
		assert_eq!(unescape_param_value("^'"), "\"");
		assert_eq!(unescape_param_value("^n"), "\n");
		assert_eq!(unescape_param_value("^N"), "\n");
		assert_eq!(unescape_param_value("^m"), "^m");
		assert_eq!(unescape_param_value("^^n"), "^n");
		assert_eq!(unescape_param_value("^^^'"), "^\"");
		assert_eq!(unescape_param_value("^^^'^^^n^'^^N^"), "^\"^\n\"^N^");
		assert_eq!(unescape_param_value("^^^^"), "^^");
		assert_eq!(unescape_param_value("^^^^n"), "^^n");
		assert_eq!(unescape_param_value("^^^^^n"), "^^\n");
	}
}

