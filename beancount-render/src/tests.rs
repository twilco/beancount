use crate::render;
use beancount_parser::parse;
use indoc::indoc;

fn test_conversion(s: &str) -> anyhow::Result<()> {
    // First obtain the ledger
    let ledger = parse(s).unwrap();

    // Now render it
    let mut rendered = Vec::new();
    render(&mut rendered, &ledger)?;
    let rendered = String::from_utf8(rendered).unwrap();

    // Parse again
    let ledger_2 = parse(&rendered).unwrap();

    // Render to test for equality
    let mut rendered_2 = Vec::new();
    render(&mut rendered_2, &ledger_2)?;
    let rendered_2 = String::from_utf8(rendered_2).unwrap();

    // Check for equality
    assert_eq!(rendered_2, rendered);

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
fn test_balance_directive() -> anyhow::Result<()> {
    test_conversion("2012-01-01 balance Assets:Checking 100 EUR\n")?;
    test_conversion("2012-01-01 balance Assets:Checking 100 ~ 1 EUR\n")?;
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
fn test_note() -> anyhow::Result<()> {
    test_conversion("2013-11-03 note Liabilities:CreditCard \"Called about fraudulent card.\"\n")?;
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
    test_conversion(indoc! {r#"
        2020-10-01 * "Sell"
          Assets:Trading             -1 HOOL {500.00 USD} @ 585.00 USD
          Assets:Trading         585.00 USD
          Income:Trading:Gains
    "#})?;
    Ok(())
}
