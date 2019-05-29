use std::collections::HashMap;

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser as PestParser;

use crate::constructs as bc;

#[derive(PestParser)]
#[grammar = "beancount.pest"]
pub struct BeancountParser;

#[derive(Debug)]
struct ParseState<'i> {
    root_names: HashMap<bc::AccountType, &'i str>,
}

pub fn parse<'i>(input: &'i str) -> bc::Ledger<'i> {
    let now = std::time::Instant::now();
    let parsed = BeancountParser::parse(Rule::file, &input)
        .expect("successful parse")
        .next()
        .unwrap();

    let mut state = ParseState {
        root_names: [
            (bc::AccountType::Assets, "Assets"),
            (bc::AccountType::Liabilities, "Liabilities"),
            (bc::AccountType::Equity, "Equity"),
            (bc::AccountType::Income, "Income"),
            (bc::AccountType::Expenses, "Expenses"),
        ]
        .iter()
        .cloned()
        .collect(),
    };

    let mut directives = Vec::new();

    for directive_pair in parsed.into_inner() {
        let dir = directive(directive_pair, &state);
        match dir {
            bc::Directive::Option(ref opt) if opt.name == "name_assets" => {
                state.root_names.insert(bc::AccountType::Assets, opt.val);
            }
            bc::Directive::Option(ref opt) if opt.name == "name_liabilities" => {
                state
                    .root_names
                    .insert(bc::AccountType::Liabilities, opt.val);
            }
            bc::Directive::Option(ref opt) if opt.name == "name_equity" => {
                state.root_names.insert(bc::AccountType::Equity, opt.val);
            }
            bc::Directive::Option(ref opt) if opt.name == "name_income" => {
                state.root_names.insert(bc::AccountType::Income, opt.val);
            }
            bc::Directive::Option(ref opt) if opt.name == "name_expenses" => {
                state.root_names.insert(bc::AccountType::Expenses, opt.val);
            }
            _ => {}
        }
        directives.push(dir);
    }
    println!("Parsing time: {:?}", now.elapsed());

    bc::Ledger::builder().directives(directives).build()
}

fn directive<'i>(directive: Pair<'i, Rule>, state: &ParseState<'i>) -> bc::Directive<'i> {
    match directive.as_rule() {
        Rule::option => option_directive(directive),
        Rule::plugin => plugin_directive(directive),
        Rule::custom => custom_directive(directive),
        Rule::include => include_directive(directive),
        Rule::open => open_directive(directive, state),
        Rule::close => close_directive(directive, state),
        Rule::commodity_directive => commodity_directive(directive),
        Rule::note => note_directive(directive, state),
        Rule::pad => pad_directive(directive, state),
        Rule::query => query_directive(directive),
        Rule::event => event_directive(directive),
        Rule::document => document_directive(directive, state),
        _ => bc::Directive::Unsupported,
    }
}

fn option_directive<'i>(directive: Pair<'i, Rule>) -> bc::Directive<'i> {
    let mut args = directive.into_inner();
    bc::Directive::Option(
        bc::BcOption::builder()
            .name(args.next().map(get_quoted_str).unwrap())
            .val(args.next().map(get_quoted_str).unwrap())
            .build(),
    )
}

fn plugin_directive<'i>(directive: Pair<'i, Rule>) -> bc::Directive<'i> {
    let mut args = directive.into_inner();
    bc::Directive::Plugin(
        bc::Plugin::builder()
            .module(args.next().map(get_quoted_str).unwrap())
            .config(args.next().map(get_quoted_str))
            .build(),
    )
}

fn custom_directive<'i>(directive: Pair<'i, Rule>) -> bc::Directive<'i> {
    let mut args = directive.into_inner();
    let date = args.next().map(|p| p.as_str()).unwrap();
    let name = args.next().map(get_quoted_str).unwrap();
    let custom_args = if args.peek().unwrap().as_rule() == Rule::custom_value_list {
        args.next()
            .unwrap()
            .into_inner()
            .map(get_quoted_str)
            .collect()
    } else {
        vec![]
    };
    let meta = meta_kv(args.next().unwrap());
    bc::Directive::Custom(
        bc::Custom::builder()
            .date(name)
            .name(date)
            .args(custom_args)
            .meta(meta)
            .build(),
    )
}

fn include_directive<'i>(directive: Pair<'i, Rule>) -> bc::Directive<'i> {
    let mut args = directive.into_inner();
    bc::Directive::Include(
        bc::Include::builder()
            .filename(args.next().map(get_quoted_str).unwrap())
            .build(),
    )
}

fn open_directive<'i>(directive: Pair<'i, Rule>, state: &ParseState<'i>) -> bc::Directive<'i> {
    let mut args = directive.into_inner();
    let date = args.next().map(|p| p.as_str()).unwrap();
    let acc = args.next().map(|p| account(p, state)).unwrap();
    let comm = if args.peek().unwrap().as_rule() == Rule::commodity_list {
        args.next()
            .map(|p| p.into_inner().map(|p| p.as_str()).collect::<Vec<_>>())
            .unwrap()
    } else {
        vec![]
    };
    let meta = args.next().map(meta_kv).unwrap();

    bc::Directive::Open(
        bc::Open::builder()
            .date(date)
            .account(acc)
            .constraint_commodities(comm)
            .meta(meta)
            .build(),
    )
}

fn close_directive<'i>(directive: Pair<'i, Rule>, state: &ParseState<'i>) -> bc::Directive<'i> {
    let mut args = directive.into_inner();
    let date = args.next().map(|p| p.as_str()).unwrap();
    let acc = args.next().map(|p| account(p, state)).unwrap();
    let meta = args.next().map(meta_kv).unwrap();
    bc::Directive::Close(
        bc::Close::builder()
            .date(date)
            .account(acc)
            .meta(meta)
            .build(),
    )
}

fn commodity_directive<'i>(directive: Pair<'i, Rule>) -> bc::Directive<'i> {
    let mut args = directive.into_inner();
    let date = args.next().map(|p| p.as_str()).unwrap();
    let name = args.next().map(|p| p.as_str()).unwrap();
    let meta = args.next().map(meta_kv).unwrap();
    bc::Directive::Commodity(
        bc::Commodity::builder()
            .date(date)
            .name(name)
            .meta(meta)
            .build(),
    )
}

fn note_directive<'i>(directive: Pair<'i, Rule>, state: &ParseState<'i>) -> bc::Directive<'i> {
    let mut args = directive.into_inner();
    let date = args.next().map(|p| p.as_str()).unwrap();
    let acc = args.next().map(|p| account(p, state)).unwrap();
    let desc = args.next().map(|p| p.as_str()).unwrap();
    let meta = args.next().map(meta_kv).unwrap();
    bc::Directive::Note(
        bc::Note::builder()
            .date(date)
            .account(acc)
            .desc(desc)
            .meta(meta)
            .build(),
    )
}

fn pad_directive<'i>(directive: Pair<'i, Rule>, state: &ParseState<'i>) -> bc::Directive<'i> {
    let mut args = directive.into_inner();
    let date = args.next().map(|p| p.as_str()).unwrap();
    let to_acc = args.next().map(|p| account(p, state)).unwrap();
    let from_acc = args.next().map(|p| account(p, state)).unwrap();
    let meta = args.next().map(meta_kv).unwrap();
    bc::Directive::Pad(
        bc::Pad::builder()
            .date(date)
            .pad_to_account(to_acc)
            .pad_from_account(from_acc)
            .meta(meta)
            .build(),
    )
}

fn query_directive<'i>(directive: Pair<'i, Rule>) -> bc::Directive<'i> {
    let mut args = directive.into_inner();
    let date = args.next().map(|p| p.as_str()).unwrap();
    let name = args.next().map(get_quoted_str).unwrap();
    let query = args.next().map(get_quoted_str).unwrap();
    let meta = args.next().map(meta_kv).unwrap();
    bc::Directive::Query(
        bc::Query::builder()
            .date(date)
            .name(name)
            .query(query)
            .meta(meta)
            .build(),
    )
}

fn event_directive<'i>(directive: Pair<'i, Rule>) -> bc::Directive<'i> {
    let mut args = directive.into_inner();
    let date = args.next().map(|p| p.as_str()).unwrap();
    let name = args.next().map(get_quoted_str).unwrap();
    let val = args.next().map(get_quoted_str).unwrap();
    let meta = args.next().map(meta_kv).unwrap();
    bc::Directive::Event(
        bc::Event::builder()
            .date(date)
            .name(name)
            .val(val)
            .meta(meta)
            .build(),
    )
}

fn document_directive<'i>(directive: Pair<'i, Rule>, state: &ParseState<'i>) -> bc::Directive<'i> {
    let mut args = directive.into_inner();
    let date = args.next().map(|p| p.as_str()).unwrap();
    let account = args.next().map(|p| account(p, state)).unwrap();
    let path = args.next().map(get_quoted_str).unwrap();
    let meta = args.next().map(meta_kv).unwrap();
    bc::Directive::Document(
        bc::Document::builder()
            .date(date)
            .account(account)
            .path(path)
            .meta(meta)
            .build(),
    )
}

fn account<'i>(pair: Pair<'i, Rule>, state: &ParseState<'i>) -> bc::Account<'i> {
    debug_assert!(pair.as_rule() == Rule::account);
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap().as_str();
    let account_type = state
        .root_names
        .iter()
        .filter(|(_, &v)| v == first)
        .map(|(k, _)| k.clone())
        .next()
        .expect("invalid root account");
    let parts: Vec<_> = inner.map(|p| &p.as_str()[1..]).collect();
    bc::Account::builder().ty(account_type).parts(parts).build()
}

fn meta_kv<'i>(pair: Pair<'i, Rule>) -> HashMap<&'i str, &'i str> {
    debug_assert!(pair.as_rule() == Rule::eol_kv_list);
    pair.into_inner()
        .map(|p| {
            let mut inner = p.into_inner();
            let key = inner.next().unwrap().as_str();
            let value = inner.next().unwrap().as_str();
            (key, value)
        })
        .collect()
}

fn get_quoted_str<'i>(pair: Pair<'i, Rule>) -> &'i str {
    debug_assert!(pair.as_rule() == Rule::quoted_str);
    pair.into_inner().next().unwrap().as_str()
}

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
    fn eol_kv_list() {
        parse_ok!(eol_kv_list, "\n key: 123\n");
        parse_ok!(eol_kv_list, "\n key: 123\n key2: 456\n");
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
    fn org_mode() {
        parse_ok!(org_mode_title, "*\n");
        parse_ok!(org_mode_title, "*  \n");
        parse_ok!(org_mode_title, "*  foo\n");
        parse_fail!(org_mode_title, "  *  foo\n");
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
        parse_ok!(posting, "Assets:Cash  200 USD");
        parse_ok!(posting, "Assets:Cash");
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
        parse_ok!(transaction, "2019-02-19*\"Foo\"\"Bar\"\n");
        parse_ok!(
            transaction,
            indoc!(
                "
        2018-12-31 * \"Initalize\"
            Passiver:Foo:Bar                                   123.45 DKK
            P Passiver:Foo:Bar                                   123.45 DKK
        "
            )
        );
        parse_ok!(
            transaction,
            indoc!(
                "
        2018-12-31 * \"Initalize\"
            ; key: 123
            Assets:Foo:Bar                                   123.45 DKK
        "
            )
        );

        parse_ok!(
            transaction,
            indoc!(
                "
        2014-05-05 txn \"Cafe Mogador\" \"Lamb tagine with wine\"
        Liabilities:CreditCard:CapitalOne         -37.45 USD
        "
            ),
            indoc!(
                "
        2014-05-05 txn \"Cafe Mogador\" \"Lamb tagine with wine\"
        "
            )
        );
    }

}
