//! Type unification algorithm.

use crate::{InferType, TypeVar};
use std::collections::HashMap;

/// Unification engine.
pub struct Unifier<'a> {
    /// Current substitutions
    substitutions: &'a HashMap<TypeVar, InferType>,
    /// New substitutions from this unification
    new_substitutions: HashMap<TypeVar, InferType>,
}

impl<'a> Unifier<'a> {
    pub fn new(substitutions: &'a HashMap<TypeVar, InferType>) -> Self {
        Self {
            substitutions,
            new_substitutions: HashMap::new(),
        }
    }

    /// Unify two types, returning new substitutions.
    pub fn unify(
        mut self,
        t1: &InferType,
        t2: &InferType,
    ) -> Result<HashMap<TypeVar, InferType>, String> {
        self.unify_inner(t1, t2)?;
        Ok(self.new_substitutions)
    }

    fn unify_inner(&mut self, t1: &InferType, t2: &InferType) -> Result<(), String> {
        let t1 = self.resolve(t1);
        let t2 = self.resolve(t2);

        match (&t1, &t2) {
            // Any unifies with anything
            (InferType::Any, _) | (_, InferType::Any) => Ok(()),

            // Same concrete types
            (InferType::Concrete(a), InferType::Concrete(b)) if a == b => Ok(()),

            // Type variables
            (InferType::Var(v), t) | (t, InferType::Var(v)) => {
                if let InferType::Var(v2) = t {
                    if v == v2 {
                        return Ok(());
                    }
                }

                // Occurs check
                if t.contains_var(*v) {
                    return Err(format!("Infinite type: {:?} occurs in {:?}", v, t));
                }

                self.new_substitutions.insert(*v, t.clone());
                Ok(())
            }

            // Function types
            (
                InferType::Function {
                    params: params1,
                    return_type: ret1,
                    ..
                },
                InferType::Function {
                    params: params2,
                    return_type: ret2,
                    ..
                },
            ) => {
                if params1.len() != params2.len() {
                    return Err(format!(
                        "Function arity mismatch: {} vs {}",
                        params1.len(),
                        params2.len()
                    ));
                }

                for (p1, p2) in params1.iter().zip(params2.iter()) {
                    self.unify_inner(p1, p2)?;
                }

                self.unify_inner(ret1, ret2)
            }

            // Generic types
            (
                InferType::Generic {
                    name: n1,
                    params: p1,
                },
                InferType::Generic {
                    name: n2,
                    params: p2,
                },
            ) => {
                if n1 != n2 {
                    return Err(format!("Type mismatch: {} vs {}", n1, n2));
                }

                if p1.len() != p2.len() {
                    return Err(format!(
                        "Type parameter count mismatch: {} vs {}",
                        p1.len(),
                        p2.len()
                    ));
                }

                for (a, b) in p1.iter().zip(p2.iter()) {
                    self.unify_inner(a, b)?;
                }

                Ok(())
            }

            // Tuple types
            (InferType::Tuple(t1), InferType::Tuple(t2)) => {
                if t1.len() != t2.len() {
                    return Err(format!("Tuple length mismatch: {} vs {}", t1.len(), t2.len()));
                }

                for (a, b) in t1.iter().zip(t2.iter()) {
                    self.unify_inner(a, b)?;
                }

                Ok(())
            }

            // Array types
            (InferType::Array(a), InferType::Array(b)) => self.unify_inner(a, b),

            // Reference types
            (
                InferType::Reference {
                    inner: i1,
                    mutable: m1,
                },
                InferType::Reference {
                    inner: i2,
                    mutable: m2,
                },
            ) => {
                if m1 != m2 {
                    return Err("Reference mutability mismatch".to_string());
                }
                self.unify_inner(i1, i2)
            }

            // Unit types
            (InferType::Unit, InferType::Unit) => Ok(()),

            // Never types
            (InferType::Never, _) | (_, InferType::Never) => Ok(()),

            // Error types
            (InferType::Error, _) | (_, InferType::Error) => Ok(()),

            // Mismatch
            _ => Err(format!("Type mismatch: {:?} vs {:?}", t1, t2)),
        }
    }

    /// Resolve a type using current substitutions.
    fn resolve(&self, ty: &InferType) -> InferType {
        match ty {
            InferType::Var(v) => {
                if let Some(resolved) = self.new_substitutions.get(v) {
                    self.resolve(resolved)
                } else if let Some(resolved) = self.substitutions.get(v) {
                    self.resolve(resolved)
                } else {
                    ty.clone()
                }
            }
            _ => ty.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_concrete() {
        let subs = HashMap::new();
        let unifier = Unifier::new(&subs);

        let int = InferType::Concrete("Int".to_string());
        assert!(unifier.unify(&int, &int).is_ok());
    }

    #[test]
    fn test_unify_var() {
        let subs = HashMap::new();
        let unifier = Unifier::new(&subs);

        let var = InferType::Var(TypeVar(0));
        let int = InferType::Concrete("Int".to_string());

        let result = unifier.unify(&var, &int).unwrap();
        assert_eq!(result.get(&TypeVar(0)), Some(&int));
    }

    #[test]
    fn test_unify_mismatch() {
        let subs = HashMap::new();
        let unifier = Unifier::new(&subs);

        let int = InferType::Concrete("Int".to_string());
        let string = InferType::Concrete("String".to_string());

        assert!(unifier.unify(&int, &string).is_err());
    }
}
