use std::{fs::read_to_string, rc::Rc};

use dialog::exec::Variant as DVariant;
use dialog::{
    ast::parse_to_ast,
    exec::{DirectExecution, DirectScript, Environment},
};
use godot::{classes::ProjectSettings, prelude::*};

use crate::singletons;
use crate::state::GlobalState;

#[derive(GodotClass)]
#[class(base=Node)]
struct DialogManager {
    script: Option<Rc<DirectScript>>,
    exec: Option<DirectExecution>,
    #[export(file)]
    script_file: GString,
    #[var]
    environment: Dictionary,
    base: Base<Node>,
}

#[godot_api]
impl INode for DialogManager {
    fn init(base: Base<Node>) -> Self {
        Self {
            script: None,
            exec: None,
            script_file: GString::new(),
            environment: Dictionary::new(),
            base,
        }
    }

    fn ready(&mut self) {
        let source = ProjectSettings::singleton().globalize_path(&self.script_file);
        let source = source.to_string();
        let source = read_to_string(source).unwrap();
        let ast = parse_to_ast(&source);
        if let Ok(ast) = ast {
            self.script = Some(Rc::new(ast.as_slice().into()));
        }
        self.ready_script();
    }
}

#[godot_api]
impl DialogManager {
    #[func]
    fn start(&mut self, label: String) -> bool {
        self.exec = None;
        if let Some(ref script) = self.script {
            self.exec = DirectExecution::start(script, &label);
            let mut game_state = singletons::game_state();
            game_state.bind_mut().change_state(GlobalState::Dialog);
            self.step();
        }
        self.exec.is_some()
    }

    #[func]
    fn is_running(&mut self) -> bool {
        self.exec.is_some()
    }

    #[func(virtual)]
    fn ready_script(&mut self) {}

    #[func(virtual)]
    fn show_text(&mut self, who: String, text: String, stops: Vec<u32>) {}

    #[func(virtual)]
    fn show_choice(
        &mut self,
        store_to: String,
        choice_names: Vec<GString>,
        choice_texts: Vec<GString>,
    ) {
    }

    #[func(virtual)]
    fn trigger(&mut self, what: String) {}

    #[func(virtual)]
    fn end_dialog(&mut self) {}

    #[func]
    fn step(&mut self) {
        let step = if let Some(ref mut exec) = self.exec {
            exec.step(&mut DictionaryEnv(&mut self.environment))
        } else {
            godot_warn!("Trying to progress absent execution!");
            return;
        };
        match step {
            dialog::exec::ExecutionStep::Text(who, text, items) => {
                let stops = items.into_iter().map(|v| v as u32).collect();
                let who = who.as_ref().map_or("", |who| who.as_str()).to_string();
                self.show_text(who, text.as_str().to_string(), stops);
            }
            dialog::exec::ExecutionStep::Choice(identifier, items) => {
                let (names, texts): (Vec<_>, Vec<_>) = items
                    .iter()
                    .map(|(name, text)| (name.as_str().to_godot(), text.as_str().to_godot()))
                    .unzip();
                self.show_choice(identifier.as_str().to_string(), names, texts);
            }
            dialog::exec::ExecutionStep::Trigger(ident) => {
                self.trigger(ident.as_str().to_string());
            }
            dialog::exec::ExecutionStep::End => {
                self.exec = None;
                self.end_dialog();
            }
        }
    }
}

struct DictionaryEnv<'a>(&'a mut Dictionary);

impl Environment for DictionaryEnv<'_> {
    fn get(&self, name: &str) -> Option<DVariant> {
        let value = self.0.get(name)?;
        match value.get_type() {
            VariantType::BOOL => Some(DVariant::Boolean(value.booleanize())),
            VariantType::INT => Some(DVariant::Int(value.to())),
            VariantType::FLOAT => todo!("Not yet!"),
            VariantType::STRING => Some(DVariant::String(value.to::<String>().into())),
            _ => {
                godot_warn!("Got unexpected type from dictionary!");
                None
            }
        }
    }

    fn set(&mut self, name: &str, value: DVariant) {
        self.0.set(
            name,
            match value {
                DVariant::String(string) => GString::from(&string as &str).to_variant(),
                DVariant::Int(value) => value.to_variant(),
                DVariant::Boolean(value) => value.to_variant(),
            },
        );
    }
}
