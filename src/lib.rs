pub mod constructs;
pub mod parsers;

///
/// 2014-02-03 open Assets:US:BofA:Checking
/// 2014-04-10 note Assets:US:BofA:Checking “Called to confirm wire transfer.”
/// 2014-05-02 balance Assets:US:BofA:Checking   154.20 USD
//pub fn parse_directives(in_str: &str) -> Vec<Directive> {
//
//}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn t() {
        use crate::constructs::*;
        println!("{}", "test");
//        let d = BcOptionBuilder::default().name("test").build().unwrap();
        let d = BcOption::builder().name("test").val("val").build();
        let mut v: Vec<&str> = Vec::new();
        v.push("US");
        let a = Account::builder().ty(AccountType::Assets).parts(Some(v)).build();
        println!("{:?}", a);
        println!("{:?}", d);
        assert_eq!(4, 4);
    }
}
