use std::rc::Rc;

use pest::{Parser, error::Error, iterators::Pair};
use pest_derive::Parser;

use crate::utils::UniquePush;

#[derive(Parser)]
#[grammar = "direct_script.pest"]
struct DirectScriptParser;

#[derive(Clone, Debug, PartialEq)]
pub struct Identifier(Rc<str>);

#[derive(Clone, Debug)]
pub struct Text(Rc<str>);

#[derive(Debug)]
pub enum Command {
    End,
    Jump(Identifier),
    Choice(Identifier, Identifier),
    Trigger(Identifier),
}

#[derive(Debug)]
pub enum Variable {
    Global(Identifier),
    Boolean(bool),
    String(Text),
    Int(i32),
}

#[derive(Debug)]
pub enum LogicOperation {
    Equal,
    NotEqual,
}

#[derive(Debug)]
pub enum Condition {
    Variable(Variable),
    Expr(Variable, LogicOperation, Variable),
}

#[derive(Debug)]
pub enum AstNode {
    Label(Identifier),
    Command(Command),
    Dialog(Option<Identifier>, Vec<Text>),
    Choices(Identifier, Vec<(Identifier, Text)>),
    LabelBlock(Identifier, Vec<AstNode>),
    IfBlock(Condition, Vec<AstNode>, Option<Vec<AstNode>>),
}

pub fn parse(source: &str) -> Result<Vec<AstNode>, Error<Rule>> {
    let pairs = DirectScriptParser::parse(Rule::direct_script, source)?;

    let mut ast_tree = Vec::new();
    let mut context = ParserContext::new();

    for pair in pairs {
        let node = match pair.as_rule() {
            Rule::label_block => build_ast_from_label_block(pair, &mut context),
            Rule::choice_decl => build_ast_from_choice_decl(pair, &mut context),
            Rule::EOI => continue,
            _ => panic!(
                "Unexpected declaration in global scope: {:?}",
                pair.as_rule()
            ),
        };
        ast_tree.push(node);
    }

    if !context.finalize() {
        dbg!(&context);
        panic!("Context isn't complete!");
    }

    Ok(ast_tree)
}

#[derive(Debug, Default)]
struct ParserContext {
    decl_labels: Vec<Identifier>,
    decl_choices: Vec<Identifier>,
    expected_labels: Vec<Identifier>,
    expected_choices: Vec<Identifier>,
    invoked_ident: Vec<Identifier>,
}

impl ParserContext {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn declare_label(&mut self, label: &Identifier) -> Result<(), ()> {
        if self.decl_labels.contains(label) {
            Err(())
        } else {
            self.decl_labels.push(label.clone());
            Ok(())
        }
    }

    fn declare_choice(&mut self, choice: &Identifier) -> Result<(), ()> {
        if self.decl_choices.contains(choice) {
            Err(())
        } else {
            self.decl_choices.push(choice.clone());
            Ok(())
        }
    }

    fn declare_invocation(&mut self, invoked: &Identifier) {
        self.invoked_ident.push_unique(invoked);
    }

    fn demand_label(&mut self, label: &Identifier) {
        self.expected_labels.push_unique(label);
    }

    fn demand_choice(&mut self, choice: &Identifier) {
        self.expected_choices.push_unique(choice);
    }

    fn finalize(&self) -> bool {
        // TODO: check if valid context
        self.expected_labels
            .iter()
            .all(|val| self.decl_labels.contains(val))
            && self
                .expected_choices
                .iter()
                .all(|val| self.decl_choices.contains(val))
    }
}

fn build_ast_from_label_block(block: Pair<'_, Rule>, context: &mut ParserContext) -> AstNode {
    let mut inner = block.into_inner();
    let main_label = inner
        .next()
        .expect("All label blocks are expected to have a name!");
    assert_eq!(
        main_label.as_rule(),
        Rule::label,
        "Label is expected to be first thing in block"
    );
    let ident: Identifier = main_label.into();
    context
        .declare_label(&ident)
        .expect("Duplicated label block!");

    let mut content = Vec::new();
    for pair in inner {
        let node = parse_block_content(pair, context);
        content.push(node);
    }
    AstNode::LabelBlock(ident, content)
}

fn build_ast_from_choice_decl(decl: Pair<'_, Rule>, context: &mut ParserContext) -> AstNode {
    let mut inner = decl.into_inner();
    let name = inner.next().unwrap();
    assert_eq!(
        name.as_rule(),
        Rule::name,
        "Choice declaration is expected to have a name!"
    );

    let name: Identifier = name.as_str().into();
    context
        .declare_choice(&name)
        .expect("Duplicated choice block!");

    let mut declared = Vec::new();
    for pair in inner {
        assert_eq!(pair.as_rule(), Rule::decl_inner);
        let mut inner = pair.into_inner();
        let name: Identifier = inner.next().unwrap().as_str().into();
        let text: Text = inner.next().unwrap().into();
        declared.push((name, text));
    }
    AstNode::Choices(name, declared)
}

fn parse_if_block(block: Pair<'_, Rule>, context: &mut ParserContext) -> AstNode {
    let mut inner = block.into_inner();
    let condition = inner.next().unwrap();
    assert_eq!(
        condition.as_rule(),
        Rule::condition,
        "If block is expected to have condition!"
    );

    let condition = {
        let mut inner = condition.into_inner();
        let (size, _) = inner.size_hint();
        match size {
            1 => Condition::Variable(parse_variable(inner.next().unwrap())),
            3 => {
                let rhs = parse_variable(inner.next().unwrap());
                let op = parse_logic_op(inner.next().unwrap());
                let lhs = parse_variable(inner.next().unwrap());
                Condition::Expr(rhs, op, lhs)
            }
            _ => panic!("Unexpected condition structure!"),
        }
    };

    let mut content = Vec::new();
    let mut else_part = None;
    for pair in inner {
        let node = match pair.as_rule() {
            Rule::else_part => {
                assert!(else_part.is_none(), "Got two else part in single if??");
                let mut else_content = Vec::new();
                for pair in pair.into_inner() {
                    let node = parse_block_content(pair, context);
                    else_content.push(node);
                }
                else_part = Some(else_content);
                continue;
            }
            _ => parse_block_content(pair, context),
        };
        content.push(node);
    }
    AstNode::IfBlock(condition, content, else_part)
}

fn parse_block_content(pair: Pair<'_, Rule>, context: &mut ParserContext) -> AstNode {
    match pair.as_rule() {
        Rule::label => parse_label(pair, context),
        Rule::dialog => parse_dialog(pair, context),
        Rule::command => parse_command(pair, context),
        Rule::if_block => parse_if_block(pair, context),
        _ => panic!("Unexpected token inside a block: {:?}", pair.as_rule()),
    }
}

fn parse_label(label: Pair<'_, Rule>, context: &mut ParserContext) -> AstNode {
    let ident: Identifier = label.into();
    context
        .declare_label(&ident)
        .expect("Detected duplicated label!");
    AstNode::Label(ident)
}

fn parse_dialog(dialog: Pair<'_, Rule>, context: &mut ParserContext) -> AstNode {
    assert_eq!(dialog.as_rule(), Rule::dialog);
    let mut name = None;
    let mut content: Vec<String> = Vec::new();
    for pair in dialog.into_inner() {
        match pair.as_rule() {
            Rule::name => name = Some(pair.as_str().into()),
            Rule::sep_line => (),
            Rule::sep_break => content.last_mut().unwrap().push('\n'),
            Rule::string => {
                let inner = pair.into_inner().next().unwrap();
                content.push(inner.as_str().to_owned());
            }
            _ => panic!("Unexpected token inside dialog: {:?}", pair.as_rule()),
        }
    }
    let content = content
        .into_iter()
        .map(|text| text.as_str().into())
        .collect();
    AstNode::Dialog(name, content)
}

fn parse_command(command: Pair<'_, Rule>, context: &mut ParserContext) -> AstNode {
    let command = command.into_inner().next().unwrap();
    let command = match command.as_rule() {
        Rule::end_command => Command::End,
        Rule::jump_command => {
            let jump_to = command.into_inner().next().unwrap().into();
            context.demand_label(&jump_to);
            Command::Jump(jump_to)
        }
        Rule::choice_command => {
            let mut inner = command.into_inner();
            let var: Identifier = inner.next().unwrap().into();
            let choice = inner.next().unwrap().into();
            context.demand_choice(&choice);
            Command::Choice(var, choice)
        }
        Rule::trigger_command => {
            let trigger_what = command.into_inner().next().unwrap().into();
            context.declare_invocation(&trigger_what);
            Command::Trigger(trigger_what)
        }
        _ => panic!("Unexpected token inside a command: {:?}", command.as_rule()),
    };
    AstNode::Command(command)
}

fn parse_variable(var: Pair<'_, Rule>) -> Variable {
    match var.as_rule() {
        Rule::boolean => {
            let value: bool = var.as_str().parse().unwrap();
            Variable::Boolean(value)
        }
        Rule::name => Variable::Global(var.into()),
        Rule::string => {
            let inner = var.into_inner().next().unwrap();
            Variable::String(inner.as_str().into())
        }
        Rule::number => {
            let number: i32 = var.as_str().parse().unwrap();
            Variable::Int(number)
        }
        _ => panic!("Unexpected token as variable: {:?}", var.as_rule()),
    }
}

fn parse_logic_op(op: Pair<'_, Rule>) -> LogicOperation {
    match op.as_str() {
        "==" => LogicOperation::Equal,
        "!=" => LogicOperation::NotEqual,
        _ => panic!("Unexpected logic operation: {}", op.as_str()),
    }
}

impl From<Pair<'_, Rule>> for Identifier {
    fn from(value: Pair<'_, Rule>) -> Self {
        let name = match value.as_rule() {
            Rule::label => value.into_inner().next().unwrap(),
            Rule::name => value,
            _ => panic!("Unexpected type for identifier: {:?}", value.as_rule()),
        };
        assert!(
            matches!(name.as_rule(), Rule::name),
            "Label don't have a name! Please check grammar file!"
        );
        Self(name.as_str().into())
    }
}

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl Identifier {
    pub fn to_text(self) -> Text {
        Text(self.0)
    }
}

impl From<Pair<'_, Rule>> for Text {
    fn from(string: Pair<'_, Rule>) -> Self {
        assert!(
            matches!(string.as_rule(), Rule::string),
            "Expected label pair as argument!"
        );
        let inner = string.into_inner().next().unwrap();
        assert!(
            matches!(inner.as_rule(), Rule::inner),
            "Label don't have a name! Please check grammar file!"
        );
        Self(inner.as_str().into())
    }
}

impl From<&str> for Text {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl Text {
    pub fn to_ident(self) -> Identifier {
        Identifier(self.0)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use pest::Parser;

    use crate::grammar::{parse, DirectScriptParser, Identifier, Rule};

    #[test]
    fn is_the_same() {
        let a: Identifier = "who!".into();
        let b: Identifier = "who!".into();
        assert_eq!(a, b);
        assert_eq!(&a, &b);
    }

    #[test]
    fn parse_example_file() {
        let source = read_to_string("./res/test.drs").unwrap();
        let _result = parse(&source).unwrap();
    }

    #[test]
    fn parse_label_block() {
        let source = "label:\n  who->\"Hello!\"\n  jump doom\n  end\n";
        let pairs = DirectScriptParser::parse(Rule::label_block, source)
            .unwrap()
            .next()
            .unwrap();
        println!("Pairs: {pairs:#?}");
    }
}
