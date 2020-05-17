use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use lazy_static::lazy_static;
use pest::iterators::{Pair, Pairs};
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;
use pest_derive::Parser as PestParser;
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;

use beancount_core as bc;

use error::{ParseError, ParseResult};

pub mod error;

macro_rules! construct {
    ( @fields, $builder:ident, $span:ident, $pairs:ident, ) => {};
    ( @fields, $builder:ident, $span:ident, $pairs:ident, $field:ident = if $rule:path $then:block else $else:block; $($rest:tt)* ) => {
        let $builder = match $pairs.peek() {
            Some(ref p) if p.as_rule() == $rule => {
                let f = $then;
                let pair = $pairs.next()
                    .ok_or_else(|| ParseError::invalid_state_with_span(stringify!($field), $span.clone()))?;
                $builder.$field(f(pair)?)
            },
            _ => $builder.$field($else),
        };
        construct!(@fields, $builder, $span, $pairs, $($rest)*)
    };
    ( @fields, $builder:ident, $span:ident, $pairs:ident, inner { $($field:tt)* } $($rest:tt)* ) => {
        let pair = $pairs.next()
            .ok_or_else(|| ParseError::invalid_state_with_span("inner pair", $span))?;
        let _span = pair.as_span();
        let mut inner = pair.into_inner();
        construct!(@fields, $builder, span, inner, $($field)*);
        construct!(@fields, $builder, $span, $pairs, $($rest)*)
    };
    ( @fields, $builder:ident, $span:ident, $pairs:ident, let $pat:pat = from $name:ident $block:block; $($rest:tt)* ) => {
        let $name = $pairs.next()
            .ok_or_else(|| ParseError::invalid_state_with_span(stringify!($pat), $span.clone()))?;
        let $pat = $block;
        construct!(@fields, $builder, $span, $pairs, $($rest)*)
    };
    ( @fields, $builder:ident, $span:ident, $pairs:ident, $field:ident ?= $f:expr; $($rest:tt)* ) => {
        let $builder = $builder.$field($pairs.next().map($f));
        construct!(@fields, $builder, $span, $pairs, $($rest)*)
    };
    ( @fields, $builder:ident, $span:ident, $pairs:ident, $field:ident := $val:expr; $($rest:tt)* ) => {
        let $builder = $builder.$field($val);
        construct!(@fields, $builder, $span, $pairs, $($rest)*)
    };
    ( @fields, $builder:ident, $span:ident, $pairs:ident, $field:ident = $f:expr; $($rest:tt)* ) => {
        let f = $f;
        let pair = $pairs.next().ok_or_else(|| ParseError::invalid_state(stringify!($field)))?;
        let $builder = $builder.$field(f(pair)?);
        construct!(@fields, $builder, $span, $pairs, $($rest)*)
    };
    ( $builder:ty : $pair:expr => { $($field:tt)* } ) => {
        {
            let builder = <$builder>::builder();
            let _span = $pair.as_span();
            let mut pairs = $pair.into_inner();
            construct!(@fields, builder, _span, pairs, $($field)*);
            builder.build()
        }
    };
}

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = PrecClimber::new(vec![
        Operator::new(Rule::add, Assoc::Left) | Operator::new(Rule::subtract, Assoc::Left),
        Operator::new(Rule::multiply, Assoc::Left) | Operator::new(Rule::divide, Assoc::Left),
    ]);
}

#[derive(PestParser)]
#[grammar = "beancount.pest"]
pub struct BeancountParser;

#[derive(Debug)]
struct ParseState {
    root_names: HashMap<bc::AccountType, String>,
}

fn optional_rule<'i>(rule: Rule, pairs: &mut Pairs<'i, Rule>) -> Option<Pair<'i, Rule>> {
    match pairs.peek() {
        Some(ref p) if p.as_rule() == rule => pairs.next(),
        _ => None,
    }
}

pub fn parse<'i>(input: &'i str) -> ParseResult<bc::Ledger<'i>> {
    let parsed = BeancountParser::parse(Rule::file, &input)?
        .next()
        .ok_or_else(|| ParseError::invalid_state("non-empty parse result"))?;

    let mut state = ParseState {
        root_names: [
            (bc::AccountType::Assets, "Assets".to_string()),
            (bc::AccountType::Liabilities, "Liabilities".to_string()),
            (bc::AccountType::Equity, "Equity".to_string()),
            (bc::AccountType::Income, "Income".to_string()),
            (bc::AccountType::Expenses, "Expenses".to_string()),
        ]
        .iter()
        .cloned()
        .collect(),
    };

    let mut directives = Vec::new();

    for directive_pair in parsed.into_inner() {
        if directive_pair.as_rule() == Rule::EOI {
            break;
        }
        let dir = directive(directive_pair, &state)?;
        match dir {
            bc::Directive::Option(ref opt) if opt.name == "name_assets" => {
                state
                    .root_names
                    .insert(bc::AccountType::Assets, opt.val.to_string());
            }
            bc::Directive::Option(ref opt) if opt.name == "name_liabilities" => {
                state
                    .root_names
                    .insert(bc::AccountType::Liabilities, opt.val.to_string());
            }
            bc::Directive::Option(ref opt) if opt.name == "name_equity" => {
                state
                    .root_names
                    .insert(bc::AccountType::Equity, opt.val.to_string());
            }
            bc::Directive::Option(ref opt) if opt.name == "name_income" => {
                state
                    .root_names
                    .insert(bc::AccountType::Income, opt.val.to_string());
            }
            bc::Directive::Option(ref opt) if opt.name == "name_expenses" => {
                state
                    .root_names
                    .insert(bc::AccountType::Expenses, opt.val.to_string());
            }
            _ => {}
        }
        directives.push(dir);
    }

    Ok(bc::Ledger::builder().directives(directives).build())
}

fn directive<'i>(directive: Pair<'i, Rule>, state: &ParseState) -> ParseResult<bc::Directive<'i>> {
    let dir = match directive.as_rule() {
        Rule::option => option_directive(directive)?,
        Rule::plugin => plugin_directive(directive)?,
        Rule::custom => custom_directive(directive)?,
        Rule::include => include_directive(directive)?,
        Rule::open => open_directive(directive, state)?,
        Rule::close => close_directive(directive, state)?,
        Rule::commodity_directive => commodity_directive(directive)?,
        Rule::note => note_directive(directive, state)?,
        Rule::pad => pad_directive(directive, state)?,
        Rule::query => query_directive(directive)?,
        Rule::event => event_directive(directive)?,
        Rule::document => document_directive(directive, state)?,
        Rule::price => price_directive(directive)?,
        Rule::transaction => transaction_directive(directive, state)?,
        _ => bc::Directive::Unsupported,
    };
    Ok(dir)
}

fn option_directive<'i>(directive: Pair<'i, Rule>) -> ParseResult<bc::Directive<'i>> {
    let source = directive.as_str();
    Ok(bc::Directive::Option(construct! {
        bc::BcOption: directive => {
            name = get_quoted_str;
            val = get_quoted_str;
            source := Some(source);
        }
    }))
}

fn plugin_directive<'i>(directive: Pair<'i, Rule>) -> ParseResult<bc::Directive<'i>> {
    let source = directive.as_str();
    Ok(bc::Directive::Plugin(construct! {
        bc::Plugin: directive => {
            module = get_quoted_str;
            config = get_quoted_str;
            source := Some(source);
        }
    }))
}

fn custom_directive<'i>(directive: Pair<'i, Rule>) -> ParseResult<bc::Directive<'i>> {
    let source = directive.as_str();
    Ok(bc::Directive::Custom(construct! {
        bc::Custom: directive => {
            date = date;
            name = get_quoted_str;
            args = if Rule::custom_value_list {
                |p: Pair<'i, _>| -> ParseResult<Vec<Cow<'i, str>>> {
                    p.into_inner().map(get_quoted_str).collect()
                }
            } else {
                Vec::new()
            };
            meta = meta_kv;
            source := Some(source);
        }
    }))
}

fn include_directive<'i>(directive: Pair<'i, Rule>) -> ParseResult<bc::Directive<'i>> {
    let source = directive.as_str();
    Ok(bc::Directive::Include(construct! {
        bc::Include: directive => {
            filename = get_quoted_str;
            source := Some(source);
        }
    }))
}

fn open_directive<'i>(
    directive: Pair<'i, Rule>,
    state: &ParseState,
) -> ParseResult<bc::Directive<'i>> {
    let source = directive.as_str();
    Ok(bc::Directive::Open(construct! {
        bc::Open: directive => {
            date = date;
            account = |p| account(p, state);
            currencies = if Rule::commodity_list {
                |p: Pair<'i, _>| -> ParseResult<Vec<_>> {
                    Ok(p.into_inner()
                        .map(|p| p.as_str().into())
                        .collect())
                }
            } else {
                Vec::new()
            };
            meta = meta_kv;
            source := Some(source);
        }
    }))
}

fn close_directive<'i>(
    directive: Pair<'i, Rule>,
    state: &ParseState,
) -> ParseResult<bc::Directive<'i>> {
    let source = directive.as_str();
    Ok(bc::Directive::Close(construct! {
        bc::Close: directive => {
            date = date;
            account = |p| account(p, state);
            meta = meta_kv;
            source := Some(source);
        }
    }))
}

fn commodity_directive<'i>(directive: Pair<'i, Rule>) -> ParseResult<bc::Directive<'i>> {
    let source = directive.as_str();
    Ok(bc::Directive::Commodity(construct! {
        bc::Commodity: directive => {
            date = date;
            name = as_str;
            meta = meta_kv;
            source := Some(source);
        }
    }))
}

fn note_directive<'i>(
    directive: Pair<'i, Rule>,
    state: &ParseState,
) -> ParseResult<bc::Directive<'i>> {
    let source = directive.as_str();
    Ok(bc::Directive::Note(construct! {
        bc::Note: directive => {
            date = date;
            account = |p| account(p, state);
            comment = as_str;
            meta = meta_kv;
            source := Some(source);
        }
    }))
}

fn pad_directive<'i>(
    directive: Pair<'i, Rule>,
    state: &ParseState,
) -> ParseResult<bc::Directive<'i>> {
    let source = directive.as_str();
    Ok(bc::Directive::Pad(construct! {
        bc::Pad: directive => {
            date = date;
            pad_to_account = |p| account(p, state);
            pad_from_account = |p| account(p, state);
            meta = meta_kv;
            source := Some(source);
        }
    }))
}

fn query_directive<'i>(directive: Pair<'i, Rule>) -> ParseResult<bc::Directive<'i>> {
    let source = directive.as_str();
    Ok(bc::Directive::Query(construct! {
        bc::Query: directive => {
            date = date;
            name = get_quoted_str;
            query_string = get_quoted_str;
            meta = meta_kv;
            source := Some(source);
        }
    }))
}

fn event_directive<'i>(directive: Pair<'i, Rule>) -> ParseResult<bc::Directive<'i>> {
    let source = directive.as_str();
    Ok(bc::Directive::Event(construct! {
        bc::Event: directive => {
            date = date;
            name = get_quoted_str;
            description = get_quoted_str;
            meta = meta_kv;
            source := Some(source);
        }
    }))
}

fn document_directive<'i>(
    directive: Pair<'i, Rule>,
    state: &ParseState,
) -> ParseResult<bc::Directive<'i>> {
    let source = directive.as_str();
    Ok(bc::Directive::Document(construct! {
        bc::Document: directive => {
            date = date;
            account = |p| account(p, state);
            path = get_quoted_str;
            meta = meta_kv;
            source := Some(source);
        }
    }))
}

fn price_directive<'i>(directive: Pair<'i, Rule>) -> ParseResult<bc::Directive<'i>> {
    let source = directive.as_str();
    Ok(bc::Directive::Price(construct! {
        bc::Price: directive => {
            date = date;
            currency = as_str;
            amount = amount;
            meta = meta_kv;
            source := Some(source);
        }
    }))
}

fn transaction_directive<'i>(
    directive: Pair<'i, Rule>,
    state: &ParseState,
) -> ParseResult<bc::Directive<'i>> {
    let source = directive.as_str();
    Ok(bc::Directive::Transaction(construct! {
        bc::Transaction: directive => {
            date = date;
            flag = flag;
            let (payee, narration) = from pair {
                let span = pair.as_span();
                let mut inner = pair.into_inner();
                let first = inner.next().map(get_quoted_str)
                    .transpose()?
                    .ok_or_else(|| ParseError::invalid_state_with_span("payee or narration", span))?;
                let second = inner.next().map(get_quoted_str);
                if let Some(second) = second {
                    (Some(first), second?)
                } else {
                    (None, first)
                }
            };
            payee := payee;
            narration := narration;
            let (meta, tags, links, postings) = from pair {
                let mut postings: Vec<bc::Posting<'i>> = Vec::new();
                let mut tx_meta = bc::Meta::new();
                let mut tx_tags = HashSet::new();
                let mut tx_links = HashSet::new();
                for p in pair.into_inner() {
                    match p.as_rule() {
                        Rule::posting => {
                            postings.push(posting(p, state)?);
                        }
                        Rule::key_value => {
                            let (k, v) = meta_kv_pair(p)?;
                            if let Some(last) = postings.last_mut() {
                                last.meta.insert(k, v);
                            } else {
                                tx_meta.insert(k, v);
                            }
                        }
                        Rule::tag => {
                            let tag = (&p.as_str()[1..]).into();
                            tx_tags.insert(tag);
                        }
                        Rule::link => {
                            let link = (&p.as_str()[1..]).into();
                            tx_links.insert(link);
                        }
                        rule => {
                            unimplemented!("rule {:?}", rule);
                        }
                    }
                }
                (tx_meta, tx_tags, tx_links, postings)
            };
            postings := postings;
            meta := meta;
            tags := tags;
            links := links;
            source := Some(source);
        }
    }))
}

fn posting<'i>(pair: Pair<'i, Rule>, state: &ParseState) -> ParseResult<bc::Posting<'i>> {
    let span = pair.as_span();
    let mut inner = pair.into_inner();
    let flag = optional_rule(Rule::txn_flag, &mut inner)
        .map(flag)
        .transpose()?;
    let account = inner
        .next()
        .map(|p| account(p, state))
        .transpose()?
        .ok_or_else(|| ParseError::invalid_state_with_span("account", span))?;
    let units = optional_rule(Rule::incomplete_amount, &mut inner)
        .map(incomplete_amount)
        .transpose()?
        .unwrap_or_else(|| bc::IncompleteAmount::builder().build());
    let cost = optional_rule(Rule::cost_spec, &mut inner)
        .map(cost_spec)
        .transpose()?;
    let price_anno = optional_rule(Rule::price_annotation, &mut inner)
        .map(price_annotation)
        .transpose()?;
    let price = match (price_anno, units.num) {
        (
            Some((
                true,
                bc::IncompleteAmount {
                    num: Some(n),
                    currency,
                },
            )),
            Some(n_units),
        ) => {
            let num = if n_units.is_zero() {
                0.into()
            } else {
                n / n_units.abs()
            };
            Some(
                bc::IncompleteAmount::builder()
                    .num(Some(num))
                    .currency(currency)
                    .build(),
            )
        }
        (Some((_, p)), _) => Some(p),
        (None, _) => None,
    };
    Ok(bc::Posting {
        flag,
        account,
        units,
        cost,
        price,
        meta: bc::Meta::new(),
    })
}

fn num_expr<'i>(pair: Pair<'i, Rule>) -> ParseResult<Decimal> {
    debug_assert!(pair.as_rule() == Rule::num_expr);
    PREC_CLIMBER.climb(pair.into_inner(), term, reduce_num_expr)
}

fn term<'i>(pair: Pair<'i, Rule>) -> ParseResult<Decimal> {
    debug_assert!(pair.as_rule() == Rule::term);
    let span = pair.as_span();
    let mut term_parts = pair.into_inner();
    let prefix = optional_rule(Rule::num_prefix, &mut term_parts).map(|p| p.as_str());
    let pair = term_parts
        .next()
        .ok_or_else(|| ParseError::invalid_state_with_span("num or num_expr", span))?;
    let mut num_expr = match pair.as_rule() {
        Rule::num => {
            let s = pair.as_str().replace(',', "");
            Decimal::from_str(&s).map_err(|e| ParseError::decimal_parse_error(e, pair.as_span()))?
        }
        Rule::num_expr => num_expr(pair)?,
        _ => unimplemented!(),
    };
    if let Some("-") = prefix {
        num_expr.set_sign_positive(!num_expr.is_sign_positive());
    }
    Ok(num_expr)
}

fn reduce_num_expr<'i>(
    lhs: ParseResult<Decimal>,
    op: Pair<'i, Rule>,
    rhs: ParseResult<Decimal>,
) -> ParseResult<Decimal> {
    let lhs = lhs?;
    let rhs = rhs?;
    Ok(match op.as_rule() {
        Rule::add => lhs + rhs,
        Rule::subtract => lhs - rhs,
        Rule::multiply => lhs * rhs,
        Rule::divide => lhs / rhs,
        _ => unimplemented!(),
    })
}

fn amount<'i>(pair: Pair<'i, Rule>) -> ParseResult<bc::Amount<'i>> {
    debug_assert!(pair.as_rule() == Rule::amount);
    Ok(construct! {
        bc::Amount: pair => {
            num = num_expr;
            currency = as_str;
        }
    })
}

fn incomplete_amount<'i>(pair: Pair<'i, Rule>) -> ParseResult<bc::IncompleteAmount<'i>> {
    debug_assert!(pair.as_rule() == Rule::incomplete_amount);
    Ok(construct! {
        bc::IncompleteAmount: pair => {
            num = if Rule::num_expr {
                |p| num_expr(p).map(Some)
            } else {
                None
            };
            currency = if Rule::commodity {
                |p| as_str(p).map(|s| Some(s.into()))
            } else {
                None
            };
        }
    })
}

fn cost_spec<'i>(pair: Pair<'i, Rule>) -> ParseResult<bc::CostSpec<'i>> {
    debug_assert!(pair.as_rule() == Rule::cost_spec);
    let mut amount = (None, None, None);
    let mut date_ = None;
    let mut label = None;
    let span = pair.as_span();
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| ParseError::invalid_state_with_span("cost spec component", span))?;
    let typ = inner.as_rule();
    for p in inner.into_inner() {
        match p.as_rule() {
            Rule::date => date_ = Some(date(p)?.into()),
            Rule::quoted_str => label = Some(get_quoted_str(p)?),
            Rule::compound_amount => {
                amount = compound_amount(p)?;
            }
            _ => unimplemented!(),
        }
    }
    if typ == Rule::cost_spec_total {
        if !amount.1.is_none() {
            panic!("Per-unit cost may not be specified using total cost");
        }
        amount = (None, amount.0, amount.2);
    }
    Ok(bc::CostSpec::builder()
        .number_per(amount.0)
        .number_total(amount.1)
        .currency(amount.2)
        .date(date_)
        .label(label)
        .build())
}

fn price_annotation<'i>(pair: Pair<'i, Rule>) -> ParseResult<(bool, bc::IncompleteAmount<'i>)> {
    debug_assert!(pair.as_rule() == Rule::price_annotation);
    let span = pair.as_span();
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| ParseError::invalid_state_with_span("price annotation", span.clone()))?;
    let is_total = inner.as_rule() == Rule::price_annotation_total;
    let amount = incomplete_amount(
        inner
            .into_inner()
            .next()
            .ok_or_else(|| ParseError::invalid_state_with_span("incomplete amount", span))?,
    )?;
    Ok((is_total, amount))
}

fn account<'i>(pair: Pair<'i, Rule>, state: &ParseState) -> ParseResult<bc::Account<'i>> {
    debug_assert!(pair.as_rule() == Rule::account);
    let span = pair.as_span();
    let mut inner = pair.into_inner();
    let first_pair = inner
        .next()
        .ok_or_else(|| ParseError::invalid_state_with_span("first part of account name", span))?;
    let first = first_pair.as_str();
    let account_type = state
        .root_names
        .iter()
        .filter(|(_, ref v)| *v == first)
        .map(|(k, _)| k.clone())
        .next()
        .ok_or_else(|| {
            pest::error::Error::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: "Invalid root account".to_string(),
                },
                first_pair.as_span(),
            )
        })?;
    let parts: Vec<_> = inner.map(|p| Cow::Borrowed(&p.as_str()[1..])).collect();
    Ok(bc::Account::builder().ty(account_type).parts(parts).build())
}

fn as_str<'i>(pair: Pair<'i, Rule>) -> ParseResult<&'i str> {
    Ok(pair.as_str())
}

fn date<'i>(pair: Pair<'i, Rule>) -> ParseResult<&'i str> {
    Ok(pair.as_str())
}

fn meta_kv<'i>(pair: Pair<'i, Rule>) -> ParseResult<HashMap<&'i str, &'i str>> {
    debug_assert!(pair.as_rule() == Rule::eol_kv_list);
    pair.into_inner().map(meta_kv_pair).collect()
}

fn meta_kv_pair<'i>(pair: Pair<'i, Rule>) -> ParseResult<(&'i str, &'i str)> {
    debug_assert!(pair.as_rule() == Rule::key_value);
    let span = pair.as_span();
    let mut inner = pair.into_inner();
    let key = inner
        .next()
        .ok_or_else(|| ParseError::invalid_state_with_span("metadata key", span.clone()))?
        .as_str();
    let value = inner
        .next()
        .ok_or_else(|| ParseError::invalid_state_with_span("metadata value", span))?
        .as_str();
    Ok((key, value))
}

fn get_quoted_str<'i>(pair: Pair<'i, Rule>) -> ParseResult<Cow<'i, str>> {
    debug_assert!(pair.as_rule() == Rule::quoted_str);
    let span = pair.as_span();
    Ok(pair
        .into_inner()
        .next()
        .ok_or_else(|| ParseError::invalid_state_with_span("quoted string", span))?
        .as_str()
        .into())
}

fn flag<'i>(pair: Pair<'i, Rule>) -> ParseResult<bc::Flag> {
    Ok(bc::Flag::from(pair.as_str()))
}

fn compound_amount<'i>(
    pair: Pair<'i, Rule>,
) -> ParseResult<(Option<Decimal>, Option<Decimal>, Option<Cow<'i, str>>)> {
    let mut number_per = None;
    let mut number_total = None;
    let mut currency = None;
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::num_expr => {
                let num = Some(num_expr(p)?);
                if number_per.is_none() {
                    number_per = num;
                } else {
                    number_total = num;
                }
            }
            Rule::commodity => {
                currency = Some(p.as_str().into());
            }
            _ => unimplemented!(),
        }
    }
    Ok((number_per, number_total, currency))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bc;
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
        parse_ok!(num, "1.2");
        parse_ok!(num, "1.2");
        parse_ok!(num, "1.2");
        parse_ok!(num, "1000.2");
        parse_ok!(num, "1,000.2");
        parse_ok!(num, "1,222,333.4");

        parse_ok!(num, "1234,0", "1234");
        parse_ok!(num, "1,1234", "1,123");
        parse_ok!(num, "1,222,33.4", "1,222");
    }

    #[test]
    fn num_expr() {
        parse_ok!(num_expr, "1");
        parse_ok!(num_expr, "+1");
        parse_ok!(num_expr, "-1");
        parse_ok!(num_expr, "-(1)");
        parse_ok!(num_expr, "+(1)");
        parse_ok!(num_expr, "1 + 2");
        parse_ok!(num_expr, "1 - 2");
        parse_ok!(num_expr, "1 * 2");
        parse_ok!(num_expr, "1 / 2");
        parse_ok!(num_expr, "1+-(2*3)/(4+6)");

        parse_ok!(num_expr, "1+-+(1)", "1");
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

        // DEPRECATED PIPE SYNTAX
        parse_ok!(
            transaction,
            indoc!(
                "
        2014-05-05 txn \"Cafe Mogador\" | \"Lamb tagine with wine\"
        Liabilities:CreditCard:CapitalOne         -37.45 USD
        "
            ),
            indoc!(
                "
        2014-05-05 txn \"Cafe Mogador\" | \"Lamb tagine with wine\"
        "
            )
        );

        let source = indoc!(
            "
            2014-05-05 txn \"Cafe Mogador\" \"Lamb tagine with wine\"
                Liabilities:CreditCard:CapitalOne         10 USD { 15 GBP } @ 20 GBP
            "
        );
        assert_eq!(
            parse(&source).unwrap(),
            bc::Ledger {
                directives: vec![bc::Directive::Transaction(
                    bc::Transaction::builder()
                        .date("2014-05-05")
                        .payee(Some("Cafe Mogador".into()))
                        .narration("Lamb tagine with wine")
                        .postings(vec![bc::Posting::builder()
                            .account(
                                bc::Account::builder()
                                    .ty(bc::AccountType::Liabilities)
                                    .parts(vec!["CreditCard".into(), "CapitalOne".into()])
                                    .build()
                            )
                            .units(
                                bc::IncompleteAmount::builder()
                                    .num(Some(10.into()))
                                    .currency(Some("USD".into()))
                                    .build()
                            )
                            .cost(
                                bc::CostSpec::builder()
                                    .number_per(Some(15.into()))
                                    .currency(Some("GBP".into()))
                                    .build()
                            )
                            .price(Some(
                                bc::IncompleteAmount::builder()
                                    .num(Some(20.into()))
                                    .currency(Some("GBP".into()))
                                    .build()
                            ))
                            .build()])
                        .source(source)
                        .build()
                )]
            }
        )
    }
}
