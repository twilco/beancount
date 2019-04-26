use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "beancount.pest"]
pub struct BeancountParser;

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pest::Parser;

    macro_rules! parse_ok {
        ( $rule:ident, $input:expr ) => {
            assert_eq!(
                BeancountParser::parse(Rule::$rule, $input)
                    .unwrap()
                    .as_str(),
                $input
            );
        };
        ( $rule:ident, $input:expr, $output:expr ) => {
            assert_eq!(
                BeancountParser::parse(Rule::$rule, $input)
                    .unwrap()
                    .as_str(),
                $output
            );
        };
    }

    macro_rules! parse_fail {
        ( $rule:ident, $input:expr ) => {
            assert!(BeancountParser::parse(Rule::$rule, $input).is_err());
        };
    }

    #[test]
    fn key_value() {
        parse_ok!(key_value, "key: \"value\"");
        parse_ok!(key_value, "key:\"value\"");
        parse_ok!(key_value, "key:    \"value\"");
        parse_ok!(key_value, "key: Assets:Cash");
        parse_ok!(key_value, "key: 2019-01-01");
        parse_ok!(key_value, "key: USD");
        parse_ok!(key_value, "key: #foo");
        parse_ok!(key_value, "key: True");
        parse_ok!(key_value, "key: 200.00 USD");
        parse_ok!(key_value, "key: 200.00");
        parse_ok!(key_value, "key1: 1");

        parse_fail!(key_value, "key    : \"value\"");
        parse_fail!(key_value, "key: bar");
        parse_fail!(key_value, "k: 123");
        parse_fail!(key_value, "Key: 123");
    }

    #[test]
    fn key_value_list() {
        parse_ok!(key_value_list, " key: 123\n");
        parse_ok!(key_value_list, " key: 123\n key2: 456\n");
    }

    #[test]
    fn date() {
        parse_ok!(date, "2019-01-12");
        parse_ok!(date, "1979/01/01");
        parse_ok!(date, "2019-12-31");

        parse_fail!(date, "123-01-01");
        parse_fail!(date, "2020-13-01");
        parse_fail!(date, "2020-12-32");
        parse_fail!(date, "2020 02 02");
        parse_fail!(date, "02-02-2020");
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

        parse_ok!(
            commodity,
            "FOOOOOOOOOOOOOOOOOOOOOOOX",
            "FOOOOOOOOOOOOOOOOOOOOOOO"
        );
        parse_ok!(
            commodity,
            "FOOOOOOOOOOOOOOOOOOOOOO.",
            "FOOOOOOOOOOOOOOOOOOOOOO"
        );
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

    #[test]
    fn tag() {
        parse_ok!(tag, "#foo");
        parse_ok!(tag, "#FOO");
        parse_ok!(tag, "#123");
        parse_ok!(tag, "#foo-123/asd.asfd_asd");
        parse_ok!(tag, "#foo bar", "#foo");
        parse_ok!(link, "^foo");

        parse_ok!(tag, "#fooæ", "#foo");
        parse_fail!(tag, "#");
    }

    #[test]
    fn balance() {
        parse_ok!(balance, "2014-08-09 balance Assets:Cash 562.00 USD\n");
        parse_ok!(
            balance,
            "2014-08-09 balance Assets:Cash 562.00 USD\n  foo: \"bar\"\n"
        );
        parse_ok!(
            balance,
            "2014-08-09   balance  Assets:Cash    562.00  USD\n"
        );
    }

    #[test]
    fn close() {
        parse_ok!(
            close,
            "2016-11-28 close Liabilities:CreditCard:CapitalOne\n"
        );
    }

    #[test]
    fn commodity_directive() {
        parse_ok!(commodity_directive, "2012-01-01 commodity HOOL\n");
    }

    #[test]
    fn custom() {
        parse_ok!(custom, "2014-07-09 custom \"budget\" \"some_config_opt_for_custom_directive\" TRUE 45.30 USD\n");
    }

    #[test]
    fn document() {
        parse_ok!(
            document,
            "2013-11-03 document Liabilities:CreditCard \"/home/joe/stmts/apr-2014.pdf\"\n"
        );
    }

    #[test]
    fn event() {
        parse_ok!(event, "2014-07-09 event \"location\" \"Paris, France\"\n");
    }

    #[test]
    fn include() {
        parse_ok!(include, "include \"path/to/include/file.beancount\"\n");
    }

    #[test]
    fn note() {
        parse_ok!(
            note,
            "2013-11-03 note Liabilities:CreditCard \"Called about fraudulent card.\"\n"
        );
    }

    #[test]
    fn open() {
        parse_ok!(
            open,
            "2014-05-01 open Liabilities:CreditCard:CapitalOne USD\n"
        );
    }

    #[test]
    fn option() {
        parse_ok!(option, "option \"title\" \"Ed’s Personal Ledger\"\n");
    }

    #[test]
    fn pad() {
        parse_ok!(
            pad,
            "2014-06-01 pad Assets:BofA:Checking Equity:Opening-Balances\n"
        );
    }

    #[test]
    fn plugin() {
        parse_ok!(
            plugin,
            "plugin \"beancount.plugins.module_name\" \"configuration data\"\n"
        );
    }

    #[test]
    fn price() {
        parse_ok!(price, "2014-07-09 price HOOL 579.18 USD\n");
    }

    #[test]
    fn query() {
        parse_ok!(query, "2014-07-09 query \"france-balances\" \"SELECT account, sum(position) WHERE ‘trip-france-2014’ in tags\"\n");
    }

    #[test]
    fn posting() {
        parse_ok!(posting, " Assets:Cash  200 USD\n");
        parse_ok!(posting, " Assets:Cash\n");
    }

    #[test]
    fn transaction() {
        parse_ok!(
            transaction,
            indoc!(
                "
            2014-05-05 txn \"Cafe Mogador\" \"Lamb tagine with wine\"
                Liabilities:CreditCard:CapitalOne         -37.45 USD
                Expenses:Restaurant
            "
            )
        );
    }

}
