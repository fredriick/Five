//! Environment/scope management for the Five interpreter.

use crate::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Binding information for a variable.
#[derive(Debug, Clone)]
pub struct Binding {
    pub value: Value,
    pub mutable: bool,
}

/// An environment (scope) containing variable bindings.
#[derive(Debug)]
pub struct Environment {
    /// Variable bindings in this scope.
    bindings: HashMap<String, Binding>,
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

    /// Define a new variable in this scope (immutable by default).
    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, Binding { value, mutable: false });
    }

    /// Define a new variable in this scope with explicit mutability.
    pub fn define_mut(&mut self, name: String, value: Value, mutable: bool) {
        self.bindings.insert(name, Binding { value, mutable });
    }

    /// Get a variable's value, searching up the scope chain.
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(binding) = self.bindings.get(name) {
            return Some(binding.value.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }

        None
    }

    /// Check if a variable is mutable.
    pub fn is_mutable(&self, name: &str) -> Option<bool> {
        if let Some(binding) = self.bindings.get(name) {
            return Some(binding.mutable);
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().is_mutable(name);
        }

        None
    }

    /// Set a variable's value, searching up the scope chain.
    /// Returns Ok(true) if set successfully, Err if not mutable, Ok(false) if not found.
    pub fn set(&mut self, name: &str, value: Value) -> Result<bool, &'static str> {
        if let Some(binding) = self.bindings.get_mut(name) {
            if !binding.mutable {
                return Err("Cannot assign to immutable variable");
            }
            binding.value = value;
            return Ok(true);
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().set(name, value);
        }

        Ok(false)
    }

    /// Set a variable's value without checking mutability (for built-ins).
    pub fn set_unchecked(&mut self, name: &str, value: Value) -> bool {
        if let Some(binding) = self.bindings.get_mut(name) {
            binding.value = value;
            return true;
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().set_unchecked(name, value);
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
