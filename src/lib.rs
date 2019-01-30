mod parser;

mod helper;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
        let s="àǜòŋäïu╭";
        println!("'{}':{}", s,s.len());
    }
}
