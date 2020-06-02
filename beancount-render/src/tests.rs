use crate::render;
use beancount_parser::parse;

fn test_conversion(s: &str) -> anyhow::Result<()> {
    // First obtain the ledger
    let ledger = parse(s).unwrap();

    // Now render it
    let mut rendered = String::new();
    render(&mut rendered, &ledger)?;

    // Parse again
    let ledger_2 = parse(dbg!(&rendered)).unwrap();

    // Check for equality
    assert_eq!(ledger_2, ledger);

    Ok(())
}

#[test]
fn test_balance() -> anyhow::Result<()> {
    test_conversion(
        "2014-08-08 open Assets:Cash
    2014-08-09 balance Assets:Cash 562.00 USD\n",
    )?;
    test_conversion(
        "2014-08-08 open Assets:Cash
    2014-08-09 balance Assets:Cash 562.00 USD\n  foo: \"bar\"\n",
    )?;
    test_conversion(
        "2014-08-08 open Assets:Cash
    2014-08-09   balance  Assets:Cash    562.00  USD\n",
    )?;
    Ok(())
}

#[test]
fn test_close() -> anyhow::Result<()> {
    test_conversion("2016-11-28 close Liabilities:CreditCard:CapitalOne\n")?;
    Ok(())
}

#[test]
fn test_commodity_directive() -> anyhow::Result<()> {
    test_conversion("2012-01-01 commodity HOOL\n")?;
    Ok(())
}

#[test]
// TODO: Failing test
#[ignore]
fn test_custom() -> anyhow::Result<()> {
    test_conversion(
        "2014-07-09 custom \"budget\" \"some_config_opt_for_custom_directive\" TRUE 45.30 USD\n",
    )?;
    Ok(())
}

#[test]
fn test_document() -> anyhow::Result<()> {
    test_conversion(
        "2013-11-03 document Liabilities:CreditCard \"/home/joe/stmts/apr-2014.pdf\"\n",
    )?;
    Ok(())
}

#[test]
fn test_event() -> anyhow::Result<()> {
    test_conversion("2014-07-09 event \"location\" \"Paris, France\"\n")?;
    Ok(())
}

#[test]
fn test_include() -> anyhow::Result<()> {
    test_conversion("include \"path/to/include/file.beancount\"\n")?;
    Ok(())
}

#[test]
fn test_note() -> anyhow::Result<()> {
    test_conversion("2013-11-03 note Liabilities:CreditCard \"Called about fraudulent card.\"\n")?;
    Ok(())
}

#[test]
fn test_open() -> anyhow::Result<()> {
    test_conversion("2014-05-01 open Liabilities:CreditCard:CapitalOne USD\n")?;
    Ok(())
}

#[test]
fn test_option() -> anyhow::Result<()> {
    test_conversion("option \"title\" \"Ed’s Personal Ledger\"\n")?;
    Ok(())
}

#[test]
fn test_pad() -> anyhow::Result<()> {
    test_conversion("2014-06-01 pad Assets:BofA:Checking Equity:Opening-Balances\n")?;
    Ok(())
}

#[test]
fn test_plugin() -> anyhow::Result<()> {
    test_conversion("plugin \"beancount.plugins.module_name\" \"configuration data\"\n")?;
    Ok(())
}

#[test]
fn test_price() -> anyhow::Result<()> {
    test_conversion("2014-07-09 price HOOL 579.18 USD\n")?;
    Ok(())
}

#[test]
fn test_query() -> anyhow::Result<()> {
    test_conversion("2014-07-09 query \"france-balances\" \"SELECT account, sum(position) WHERE ‘trip-france-2014’ in tags\"\n")?;
    Ok(())
}

#[test]
fn test_transaction() -> anyhow::Result<()> {
    test_conversion(
        "
    2014-05-05 txn \"Cafe Mogador\" \"Lamb tagine with wine\"
        Liabilities:CreditCard:CapitalOne         -37.45 USD
        Expenses:Restaurant
    ",
    )?;
    test_conversion("2019-02-19*\"Foo\"\"Bar\"\n")?;
    test_conversion(
        "
        2018-12-31 * \"Initalize\"
            Passiver:Foo:Bar                                   123.45 DKK
            P Passiver:Foo:Bar                                   123.45 DKK
        ",
    )?;
    test_conversion(
        "
            2018-12-31 * \"Initalize\"
                ; key: 123
                Assets:Foo:Bar                                   123.45 DKK
            ",
    )?;
    test_conversion(
        "
            2014-05-05 txn \"Cafe Mogador\" \"Lamb tagine with wine\"
            Liabilities:CreditCard:CapitalOne         -37.45 USD
            ",
    )?;
    Ok(())
}
