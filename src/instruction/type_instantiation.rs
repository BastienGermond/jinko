//! TypeInstantiations are used when instantiating a type. The argument list is given to the
//! type on execution.

use super::{ErrKind, FunctionDec, InstrKind, Instruction, Interpreter, JinkoError, TypeDec, Var};

use std::rc::Rc;

#[derive(Clone)]
pub struct TypeInstantiation {
    type_name: String,

    fields: Vec<Box<dyn Instruction>>,
}

impl TypeInstantiation {
    /// Create a new type instantiation and return it
    pub fn new(type_name: String) -> TypeInstantiation {
        TypeInstantiation {
            type_name,
            fields: Vec::new(),
        }
    }

    /// Add an argument to the given type instantiation
    pub fn add_field(&mut self, arg: Box<dyn Instruction>) {
        self.fields.push(arg)
    }

    /// Return a reference the called function's name
    pub fn name(&self) -> &str {
        &self.type_name
    }

    /// Return a reference to the list of fields
    pub fn fields(&self) -> &Vec<Box<dyn Instruction>> {
        &self.fields
    }

    /// Get the corresponding type declaration from an interpreter
    fn get_declaration(&self, interpreter: &mut Interpreter) -> Result<Rc<TypeDec>, JinkoError> {
        match interpreter.get_type(self.name()) {
            // get_function() return a Rc, so this clones the Rc, not the FunctionDec
            Some(t) => Ok(t.clone()),
            // FIXME: Fix Location and input
            None => Err(JinkoError::new(
                ErrKind::Interpreter,
                format!("Cannot find type {}", self.name()),
                None,
                self.name().to_owned(),
            )),
        }
    }

    /// Check if the fields received and the fields expected match
    fn check_fields_count(&self, type_dec: &TypeDec) -> Result<(), JinkoError> {
        match self.fields().len() == type_dec.fields().len() {
            true => Ok(()),
            false => Err(JinkoError::new(
                ErrKind::Interpreter,
                format!(
                    "Wrong number of arguments \
                    for call to function `{}`: Expected {}, got {}",
                    self.name(),
                    type_dec.fields().len(),
                    self.fields().len()
                ),
                None,
                "".to_owned(),
                // FIXME: Add input and location
            )),
        }
    }
}

impl Instruction for TypeInstantiation {
    fn kind(&self) -> InstrKind {
        // FIXME: Add logic
        InstrKind::Expression(None)
    }

    fn print(&self) -> String {
        let mut base = format!("{}(", self.type_name);
        let mut first_arg = true;
        for arg in &self.fields {
            if !first_arg {
                base.push_str(", ");
            }

            base.push_str(&arg.print());

            first_arg = false;
        }

        format!("{})", base)
    }

    fn execute(&self, interpreter: &mut Interpreter) -> Result<InstrKind, JinkoError> {
        let type_dec = self.get_declaration(interpreter)?;

        self.check_fields_count(&type_dec)?;

        println!("Type found {:?}", type_dec);

        // todo!("Execution for type_instantiation is not yet available");

        Err(JinkoError::new(
            ErrKind::Interpreter,
            "Execution for type_instantiation is not yet available".to_string(),
            None,
            "".to_string(),
        ))
    }
}
