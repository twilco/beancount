use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "beancount.pest"]
pub struct BeancountParser;
