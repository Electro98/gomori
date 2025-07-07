use std::{
    collections::HashMap,
    num::{NonZeroU16, NonZeroU32},
    path::Path,
    rc::Rc,
};

use crate::{
    grammar::{AstNode, Identifier, Text},
    utils::UniquePush,
};

type ChoiceVariants = Rc<[(Identifier, Text)]>;

#[derive(Debug)]
pub struct DirectScript {
    code: Box<[Command]>,
    strings: Box<[Identifier]>,
    texts: Box<[(Text, [Option<NonZeroU16>; 12])]>,
    labels: Box<[(Identifier, usize)]>,
    choices: Box<[(Identifier, ChoiceVariants)]>,
    source: Option<Rc<Path>>,
}

#[derive(Debug)]
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

#[derive(Debug)]
enum Variable {
    Name(u32),
    Boolean(bool),
    Text(u32),
    Int(i32),
}

#[repr(u8)]
#[derive(Debug)]
enum LogicOperation {
    Equal,
    NotEqual,
}

#[derive(Debug)]
enum Condition {
    Var(Variable),
    Expr(Variable, LogicOperation, Variable),
}

pub struct DirectExecution {
    script: Rc<DirectScript>,
    code_ptr: usize,
    last_condition: bool,
}

#[derive(Debug)]
pub enum ExecutionStep {
    Text(Option<Identifier>, Text, Vec<usize>),
    Choice(Identifier, ChoiceVariants),
    Trigger(Identifier),
    End,
}

impl DirectExecution {
    pub fn start(script: &Rc<DirectScript>, label: &str) -> Option<DirectExecution> {
        if let Some((_, code_ptr)) = script
            .labels
            .iter()
            .find(|(item, _)| item.as_str() == label)
        {
            Some(DirectExecution {
                script: script.clone(),
                code_ptr: *code_ptr,
                last_condition: false,
            })
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Variant {
    String(Rc<str>),
    Int(i32),
    Boolean(bool),
}

impl Variant {
    fn to_bool(&self) -> bool {
        match self {
            Variant::String(string) => !string.is_empty(),
            Variant::Int(num) => *num != 0,
            Variant::Boolean(boolean) => *boolean,
        }
    }
}

pub trait Environment {
    fn get(&self, name: &str) -> Option<Variant>;
    fn set(&mut self, name: &str, value: Variant);
}

impl DirectExecution {
    pub fn step(&mut self, env: &mut dyn Environment) -> ExecutionStep {
        loop {
            let command = &self.script.code[self.code_ptr];
            match command {
                // Control Flow
                Command::Jump(jump_to) => self.code_ptr = *jump_to,
                Command::EvalCondition(condition) => {
                    self.last_condition = match condition {
                        Condition::Var(var) => self.get_variant(env, var).unwrap().to_bool(),
                        Condition::Expr(rhs, logic_op, lhs) => {
                            let rhs = self.get_variant(env, rhs).unwrap();
                            let lhs = self.get_variant(env, lhs).unwrap();
                            match logic_op {
                                LogicOperation::Equal => rhs == lhs,
                                LogicOperation::NotEqual => rhs != lhs,
                            }
                        }
                    };
                    self.code_ptr += 1;
                }
                Command::If(skip) => {
                    if self.last_condition {
                        self.code_ptr += 1;
                    } else {
                        self.code_ptr += *skip;
                    }
                }
                // User related things
                Command::Text(who, says) => {
                    let who = who.map(|who| self.script.strings[who.get() as usize].clone());
                    let (says, stops) = &self.script.texts[*says as usize];
                    let stops = stops
                        .iter()
                        .flat_map(|item| item.map(|item| item.get() as usize))
                        .collect();
                    self.code_ptr += 1;
                    return ExecutionStep::Text(who, says.clone(), stops);
                }
                Command::Choice(store_to, what) => {
                    let store_to = self.script.strings[*store_to as usize].clone();
                    let what = self.script.choices[*what as usize].1.clone();
                    self.code_ptr += 1;
                    return ExecutionStep::Choice(store_to, what);
                }
                Command::Trigger(what) => {
                    let what = self.script.strings[*what as usize].clone();
                    self.code_ptr += 1;
                    return ExecutionStep::Trigger(what);
                }
                Command::End => return ExecutionStep::End,
            }
        }
    }

    fn get_variant<'a>(&'a self, env: &'a dyn Environment, var: &Variable) -> Option<Variant> {
        match var {
            Variable::Name(ident) => {
                let ident = self.script.strings.get(*ident as usize)?;
                env.get(ident.as_str())
            }
            Variable::Boolean(value) => Some(Variant::Boolean(*value)),
            Variable::Text(text) => {
                let text = self.script.strings.get(*text as usize)?;
                Some(Variant::String(text.as_rc().clone()))
            }
            Variable::Int(value) => Some(Variant::Int(*value)),
        }
    }
}

fn construct_script_from_ast(ast_tree: &[AstNode]) -> DirectScript {
    let choices: Box<_> = ast_tree
        .iter()
        .filter_map(|node| match node {
            AstNode::Choices(ident, content) => {
                let content = content.clone().into();
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
                Variable::Text(index as u32)
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
        choices: &[(Identifier, ChoiceVariants)],
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
                let who = who.as_ref().and_then(|who| {
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
                code.push(Command::Text(who, texts.len() as u32 - 1));
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

impl Environment for HashMap<Rc<str>, Variant> {
    fn get(&self, name: &str) -> Option<Variant> {
        self.get(name).cloned()
    }

    fn set(&mut self, name: &str, value: Variant) {
        _ = self.insert(name.into(), value);
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, fs::read_to_string, rc::Rc};

    use crate::{exec::DirectExecution, grammar::parse_to_ast, interpreter::DirectScript};

    #[test]
    fn create_from_source_file() {
        let source = read_to_string("./res/test.drs").unwrap();
        let ast_tree = parse_to_ast(&source).unwrap();
        let script: Rc<DirectScript> = Rc::new(ast_tree.as_slice().into());

        let mut env = HashMap::new();
        let mut label = DirectExecution::start(&script, "label").unwrap();

        loop {
            let exec_step = label.step(&mut env);
            match exec_step {
                super::ExecutionStep::End => break,
                step => _ = dbg!(&step),
            }
        }
    }
}
