//! Five Effects - Effect tracking system for the Five programming language.

mod infer;

pub use infer::EffectChecker;

use five_ast::Effect;
use std::collections::HashSet;

/// A set of effects.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EffectSet {
    effects: HashSet<Effect>,
}

impl EffectSet {
    /// Create a new empty effect set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a pure effect set.
    pub fn pure() -> Self {
        let mut set = Self::new();
        set.effects.insert(Effect::Pure);
        set
    }

    /// Add an effect.
    pub fn add(&mut self, effect: Effect) {
        // Pure is removed if any other effect is added
        if !matches!(effect, Effect::Pure) {
            self.effects.remove(&Effect::Pure);
        }
        self.effects.insert(effect);
    }

    /// Merge with another effect set.
    pub fn merge(&mut self, other: &EffectSet) {
        for effect in &other.effects {
            self.add(effect.clone());
        }
    }

    /// Check if this set contains an effect.
    pub fn contains(&self, effect: &Effect) -> bool {
        self.effects.contains(effect)
    }

    /// Check if this is a pure effect set.
    pub fn is_pure(&self) -> bool {
        self.effects.is_empty() || (self.effects.len() == 1 && self.effects.contains(&Effect::Pure))
    }

    /// Check if this set is a subset of another.
    pub fn is_subset_of(&self, other: &EffectSet) -> bool {
        self.effects.is_subset(&other.effects)
    }

    /// Get all effects.
    pub fn effects(&self) -> impl Iterator<Item = &Effect> {
        self.effects.iter()
    }

    /// Convert to a vector.
    pub fn to_vec(&self) -> Vec<Effect> {
        self.effects.iter().cloned().collect()
    }
}

impl From<Vec<Effect>> for EffectSet {
    fn from(effects: Vec<Effect>) -> Self {
        let mut set = Self::new();
        for effect in effects {
            set.add(effect);
        }
        set
    }
}

impl FromIterator<Effect> for EffectSet {
    fn from_iter<T: IntoIterator<Item = Effect>>(iter: T) -> Self {
        let mut set = Self::new();
        for effect in iter {
            set.add(effect);
        }
        set
    }
}

impl std::fmt::Display for EffectSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_pure() {
            write!(f, "Pure")
        } else {
            let effects: Vec<_> = self
                .effects
                .iter()
                .map(|e| match e {
                    Effect::IO => "IO",
                    Effect::State => "State",
                    Effect::Async => "Async",
                    Effect::Pure => "Pure",
                    Effect::Custom(name) => name.as_str(),
                })
                .collect();
            write!(f, "{}", effects.join(", "))
        }
    }
}
