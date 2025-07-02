use std::{
    num::{NonZeroU16, NonZeroU32},
    path::Path,
    rc::Rc,
};

use crate::{
    grammar::{AstNode, Identifier, Text},
    utils::UniquePush,
};

struct DirectScript {
    code: Box<[Command]>,
    strings: Box<[Identifier]>,
    texts: Box<[(Text, [Option<NonZeroU16>; 12])]>,
    labels: Box<[(Identifier, usize)]>,
    choices: Box<[(Identifier, Box<[(Identifier, Text)]>)]>,
    source: Option<Rc<Path>>,
}

enum Command {
    /// who | what
    Text(Option<NonZeroU32>, u32),
    Jump(usize),
    Choice(u32, u32),
    Trigger(u32),
    End,
    EvalCondition(Condition),
    If(usize),
    // Else is just jump
}

enum Variable {
    Name(u32),
    Boolean(bool),
    Text(u32),
    Int(i32),
}

#[repr(u8)]
enum LogicOperation {
    Equal,
    NotEqual,
}

enum Condition {
    Var(Variable),
    Expr(Variable, LogicOperation, Variable),
}

struct DirectExecution<'a> {
    script: &'a DirectScript,
    code_ptr: usize,
    last_condition: bool,
}

impl DirectScript {
    fn start<'a>(&'a self, label: &str) -> DirectExecution<'a> {
        todo!()
    }
}

fn construct_script_from_ast(ast_tree: &[AstNode]) -> DirectScript {
    let choices: Box<_> = ast_tree
        .iter()
        .filter_map(|node| match node {
            AstNode::Choices(ident, content) => {
                let content = content.clone().into_boxed_slice();
                Some((ident.clone(), content))
            }
            _ => None,
        })
        .collect();
    let mut code = Vec::new();
    let mut strings = vec!["safe-guard: probably a bug!".into()];
    let mut texts = Vec::new();
    let mut labels = Vec::new();

    fn count_op(node: &AstNode) -> usize {
        match node {
            AstNode::Label(_) => 1,
            AstNode::Command(_) => 1,
            AstNode::Dialog(_, _) => 1,
            AstNode::Choices(_, _) => 1,
            AstNode::LabelBlock(_, nodes) => nodes.iter().map(count_op).sum(),
            AstNode::IfBlock(_, nodes, else_nods) => {
                // Adds 2 commands: condition and if
                let if_part = 2 + nodes.iter().map(count_op).sum::<usize>();
                let else_part = else_nods.as_ref().map_or(0, |nodes| {
                    // Adds new jump to avoid else part if "if" part was executed
                    1 + nodes.iter().map(count_op).sum::<usize>()
                });
                if_part + else_part
            }
        }
    }
    fn convert_var(var: &crate::grammar::Variable, strings: &mut Vec<Identifier>) -> Variable {
        match var {
            crate::grammar::Variable::Global(ident) => {
                strings.push_unique(ident);
                let index = strings.iter().position(|item| item == ident).unwrap();
                Variable::Name(index as u32)
            }
            crate::grammar::Variable::String(text) => {
                let text = text.clone().to_ident();
                strings.push_unique(&text);
                let index = strings.iter().position(|item| item == &text).unwrap();
                Variable::Name(index as u32)
            }
            crate::grammar::Variable::Boolean(value) => Variable::Boolean(*value),
            crate::grammar::Variable::Int(value) => Variable::Int(*value),
        }
    }
    fn convert(
        node: &AstNode,
        code: &mut Vec<Command>,
        strings: &mut Vec<Identifier>,
        texts: &mut Vec<(Text, [Option<std::num::NonZero<u16>>; 12])>,
        labels: &mut Vec<(Identifier, usize)>,
        choices: &[(Identifier, Box<[(Identifier, Text)]>)],
    ) {
        match node {
            AstNode::Label(ident) => {
                labels.push((ident.clone(), code.len()));
            }
            AstNode::Command(command) => {
                let command = match command {
                    crate::grammar::Command::End => Command::End,
                    crate::grammar::Command::Jump(ident) => {
                        if let Some((_, jump_to)) = labels.iter().find(|(item, _)| item == ident) {
                            Command::Jump(*jump_to)
                        } else {
                            // TODO: how we will do it?
                            todo!("Currently all jumps should be to already defined labels");
                        }
                    }
                    crate::grammar::Command::Choice(where_to, what) => {
                        strings.push_unique(where_to);
                        let where_to = strings.iter().position(|item| item == where_to).unwrap();
                        let what = choices.iter().position(|(item, _)| item == what).unwrap();
                        Command::Choice(where_to as u32, what as u32)
                    }
                    crate::grammar::Command::Trigger(what) => {
                        strings.push_unique(what);
                        let what = strings.iter().position(|item| item == what).unwrap();
                        Command::Trigger(what as u32)
                    }
                };
                code.push(command);
            }
            AstNode::Dialog(who, says) => {
                let who = who
                    .as_ref()
                    .and_then(|who| {
                        strings.push_unique(who);
                        NonZeroU32::new(strings.iter().position(|item| item == who).unwrap() as u32)
                    });
                let indexes = says
                    .iter()
                    .map(|text| text.as_str().len())
                    .scan(0, |sum, item| {
                        *sum += item;
                        Some(*sum)
                    })
                    .collect::<Vec<_>>();
                let indexes = {
                    let mut values = [None; 12];
                    for (i, val) in indexes.into_iter().enumerate().take(12) {
                        values[i] = NonZeroU16::new(val as u16);
                    }
                    values
                };
                let text: Text = says
                    .iter()
                    .map(|text| text.as_str())
                    .fold(String::new(), |a, b| a + b)
                    .as_str()
                    .into();
                texts.push((text, indexes));
                code.push(Command::Text(who, texts.len() as u32));
            }
            AstNode::IfBlock(condition, nodes, else_nodes) => {
                let cond = match condition {
                    crate::grammar::Condition::Variable(var) => {
                        let var = convert_var(var, strings);
                        Condition::Var(var)
                    }
                    crate::grammar::Condition::Expr(rhs, logic_op, lhs) => {
                        let rhs = convert_var(rhs, strings);
                        let lhs = convert_var(lhs, strings);
                        Condition::Expr(rhs, logic_op.into(), lhs)
                    }
                };
                code.push(Command::EvalCondition(cond));
                let if_part_size = nodes.iter().map(count_op).sum::<usize>();
                // because we will add 1 new jump if there is else part
                code.push(Command::If(if_part_size + else_nodes.is_some() as usize));
                nodes
                    .iter()
                    .for_each(|node| convert(node, code, strings, texts, labels, choices));
                if let Some(else_nodes) = else_nodes {
                    let else_part_size = else_nodes.iter().map(count_op).sum::<usize>();
                    code.push(Command::Jump(code.len() + else_part_size));
                    else_nodes
                        .iter()
                        .for_each(|node| convert(node, code, strings, texts, labels, choices));
                }
            }
            _ => unreachable!(),
        }
    }

    for node in ast_tree {
        let (ident, nodes) = match node {
            AstNode::Choices(_, _) => continue,
            AstNode::LabelBlock(ident, nodes) => (ident, nodes),
            _ => unreachable!(),
        };
        labels.push((ident.clone(), code.len()));
        nodes.iter().for_each(|node| {
            convert(
                node,
                &mut code,
                &mut strings,
                &mut texts,
                &mut labels,
                &choices,
            )
        });
    }

    DirectScript {
        code: code.into_boxed_slice(),
        strings: strings.into_boxed_slice(),
        texts: texts.into_boxed_slice(),
        labels: labels.into_boxed_slice(),
        choices,
        source: None,
    }
}

impl From<&[AstNode]> for DirectScript {
    fn from(value: &[AstNode]) -> Self {
        construct_script_from_ast(value)
    }
}

impl From<crate::grammar::LogicOperation> for LogicOperation {
    fn from(value: crate::grammar::LogicOperation) -> Self {
        (&value).into()
    }
}

impl From<&crate::grammar::LogicOperation> for LogicOperation {
    fn from(value: &crate::grammar::LogicOperation) -> Self {
        match value {
            crate::grammar::LogicOperation::Equal => Self::Equal,
            crate::grammar::LogicOperation::NotEqual => Self::NotEqual,
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use crate::{grammar::parse, interpreter::DirectScript};

    #[test]
    fn create_from_source_file() {
        let source = read_to_string("./res/test.drs").unwrap();
        let ast_tree = parse(&source).unwrap();
        let _script: DirectScript = ast_tree.as_slice().into();
    }
}
