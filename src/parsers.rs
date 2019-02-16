use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "beancount.pest"]
pub struct BeancountParser;


#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;


    macro_rules! parse_ok {
        ( $rule:ident, $input:expr ) => {
            assert_eq!(BeancountParser::parse(Rule::$rule, $input).unwrap().as_str(), $input);
        };
        ( $rule:ident, $input:expr, $output:expr ) => {
            assert_eq!(BeancountParser::parse(Rule::$rule, $input).unwrap().as_str(), $output);
        };
    }

    macro_rules! parse_fail {
        ( $rule:ident, $input:expr ) => {
            assert!(BeancountParser::parse(Rule::$rule, $input).is_err());
        };
    }

    #[test]
    fn num() {
        parse_ok!(num, "1");
        parse_ok!(num, "1.");
        parse_ok!(num, "1.2");
        parse_ok!(num, "+1.2");
        parse_ok!(num, "-1.2");
        parse_ok!(num, "-1.2");
        parse_ok!(num, "-1000.2");
        parse_ok!(num, "-1,000.2");
        parse_ok!(num, "-1,222,333.4");

        parse_ok!(num, "1234,0", "1234");
        parse_ok!(num, "1,1234", "1,123");
        parse_ok!(num, "1,222,33.4", "1,222");
    }

    #[test]
    fn quoted_str() {
        parse_ok!(quoted_str, r#""""#);
        parse_ok!(quoted_str, r#""foo""#);
        parse_ok!(quoted_str, r#""€☃""#);
        parse_ok!(quoted_str, r#""\"""#);
        parse_ok!(quoted_str, r#""\x""#);
        parse_ok!(quoted_str, r#"" foo ""#);
    }

    #[test]
    fn commodity() {
        parse_ok!(commodity, "AAA");
        parse_ok!(commodity, "EUR");
        parse_ok!(commodity, "FOO_BAR");
        parse_ok!(commodity, "FOO.BAR");
        parse_ok!(commodity, "FOO-BAR");
        parse_ok!(commodity, "FOO'BAR");
        parse_ok!(commodity, "F123");
        parse_ok!(commodity, "FOO-123");
        parse_ok!(commodity, "FOOOOOOOOOOOOOOOOOOOOOOO");

        parse_ok!(commodity, "FOOOOOOOOOOOOOOOOOOOOOOOX", "FOOOOOOOOOOOOOOOOOOOOOOO");
        parse_ok!(commodity, "FOOOOOOOOOOOOOOOOOOOOOO.", "FOOOOOOOOOOOOOOOOOOOOOO");
        parse_ok!(commodity, "FOO\"123", "FOO");
        parse_fail!(commodity, "123");
        parse_fail!(commodity, "foo");
    }

    #[test]
    fn account() {
        parse_ok!(account, "Assets:Foo");
        parse_ok!(account, "Indtægter:Foo");
        parse_ok!(account, "Expenses:Q1");
        parse_ok!(account, "Expenses:Tax:2018");
        parse_ok!(account, "Dash-dash:Dash-dash");

        parse_fail!(account, "Assets");
        parse_fail!(account, "Assets:");
        parse_fail!(account, "Assets: Foo");
        parse_fail!(account, "Expenses:tax");
    }
}
