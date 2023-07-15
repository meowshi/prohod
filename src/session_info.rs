use std::{fmt::Display, ops::AddAssign};

use cli_table::{format::Justify, Table};

#[derive(Default, Debug, Table)]
pub struct SessionInfo {
    #[table(title = "Время", justify = "Justify::Center")]
    pub(crate) time: String,
    #[table(title = "Продано", justify = "Justify::Center")]
    pub(crate) sold: u32,
    #[table(title = "Пришли", justify = "Justify::Center")]
    pub(crate) passed: u32,
    #[table(title = "Осталось", justify = "Justify::Center")]
    pub(crate) left: u32,
}

impl SessionInfo {
    pub fn new(sold: u32, passed: u32, left: u32) -> Self {
        SessionInfo {
            time: "".to_owned(),
            sold,
            passed,
            left,
        }
    }
}

impl Display for SessionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.time, self.sold, self.passed, self.left
        )
    }
}

impl<'a> AddAssign<&'a Self> for SessionInfo {
    fn add_assign(&mut self, rhs: &'a Self) {
        self.left += rhs.left;
        self.passed += rhs.passed;
        self.sold += rhs.sold;
    }
}
