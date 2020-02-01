use beancount::parsers::parse;

fn main() {
    let filename = std::env::args().nth(1).expect("filename argument");
    let unparsed_file = std::fs::read_to_string(filename).expect("cannot read file");

    let ledger = parse(&unparsed_file);
    dbg!(ledger);
}
