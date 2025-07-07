mod grammar;
mod interpreter;
mod utils;

// Re-exports
pub mod ast {
    pub use crate::grammar::{
        AstNode, Command, Condition, Identifier, LogicOperation, Rule, Text, Variable, parse_to_ast,
    };
}

pub mod exec {
    pub use crate::interpreter::{
        DirectExecution, DirectScript, Environment, ExecutionStep, Variant,
    };
}
