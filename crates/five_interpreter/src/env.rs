//! Environment/scope management for the Five interpreter.

use crate::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// An environment (scope) containing variable bindings.
#[derive(Debug)]
pub struct Environment {
    /// Variable bindings in this scope.
    bindings: HashMap<String, Value>,
    /// Parent environment (for lexical scoping).
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    /// Create a new global environment.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    /// Create a new environment with a parent (for nested scopes).
    pub fn with_parent(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }

    /// Define a new variable in this scope.
    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    /// Get a variable's value, searching up the scope chain.
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.bindings.get(name) {
            return Some(value.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }

        None
    }

    /// Set a variable's value, searching up the scope chain.
    pub fn set(&mut self, name: &str, value: Value) -> bool {
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_string(), value);
            return true;
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().set(name, value);
        }

        false
    }

    /// Check if a variable exists in this scope or any parent scope.
    pub fn contains(&self, name: &str) -> bool {
        if self.bindings.contains_key(name) {
            return true;
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().contains(name);
        }

        false
    }

    /// Get all variable names in this scope (not including parents).
    pub fn local_names(&self) -> Vec<String> {
        self.bindings.keys().cloned().collect()
    }
}
