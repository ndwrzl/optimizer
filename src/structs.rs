use serde::{Deserialize, Serialize};

use core::fmt;

#[derive(Serialize, Deserialize, Default)]
pub struct KillParse {
    pub services: Vec<KillService>,
    pub processes: Vec<Kill>,
}

#[derive(Default)]
pub enum Action {
    #[default]
    Kill,
    Restore,
}

#[derive(Debug, Default, PartialEq)]
pub enum Types {
    #[default]
    Process,
    Service,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::Kill => write!(f, "Kill"),
            Action::Restore => write!(f, "Restore"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Kill {
    pub name: String,
    pub restore: bool,

    #[serde(default = "_default_true")]
    pub enabled: bool,
    #[serde(default = "_default_false")]
    pub admin: bool,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct KillService {
    pub name: String,
    pub restore: bool,
    #[serde(default = "_default_true")]
    pub enabled: bool,
}

const fn _default_true() -> bool {
    true
}
const fn _default_false() -> bool {
    false
}
