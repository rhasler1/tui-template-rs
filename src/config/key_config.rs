use serde::{Serialize, Deserialize};

use crate::events::key::Key;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyConfig {
    pub exit: Key
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            exit: Key::Esc
        }
    }
}