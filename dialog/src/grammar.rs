
use pest_derive::Parser;
use pest::{error::Error, Parser};

#[derive(Parser)]
#[grammar = "direct_script.pest"]
struct DirectScriptParser;

enum Command<'a> {
    End,
    Jump(&'a str),
    Choice(&'a str, &'a str),
    Trigger(&'a str),
}

enum Script<'a> {
    Label(&'a str),
    Dialog(Option<&'a str>, Vec<&'a str>), // todo
    Command(Command<'a>),
    IfElse(Vec<Script<'a>>, Option<Vec<Script<'a>>>),
    Choices(&'a str, Vec<(&'a str, &'a str)>),
}

pub fn parse(source: &str) -> Result<(), Error<Rule>> {

    let pairs = DirectScriptParser::parse(Rule::direct_script, source)?.next().unwrap();

    println!("Pairs: {pairs:#?}");
    // dbg!(pairs);
    
    todo!()
}

#[cfg(test)]
mod test {
    use std::fs::{read_to_string};

    use pest::Parser;

    use crate::grammar::{parse, DirectScriptParser, Rule};

    #[test]
    fn parse_example_file() {
        let source = read_to_string("./res/test.drs").unwrap();
        let result = parse(&source).unwrap();
    }

    #[test]
    fn parse_label_block() {
        let source = "label:\n  who->\"Hello!\"\n  jump doom\n  end\n";
        let pairs = DirectScriptParser::parse(Rule::label_block, source).unwrap().next().unwrap();
        println!("Pairs: {pairs:#?}");
    }
}