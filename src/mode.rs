#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
}

impl Mode {
    pub fn is_normal(&self) -> bool {
        *self == Mode::Normal
    }

    pub fn is_insert(&self) -> bool {
        *self == Mode::Insert
    }
}