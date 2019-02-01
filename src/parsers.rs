use pest::Parser;

#[derive(Parser)]
#[grammar = "beancount.pest"]
pub struct BeancountParser;