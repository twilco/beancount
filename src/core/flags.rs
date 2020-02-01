#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Flag {
    Okay,
    Warning,
    Other(String),
}

impl Default for Flag {
    fn default() -> Self {
        Flag::Okay
    }
}
