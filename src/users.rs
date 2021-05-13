use std::fmt::{Display, Result, Formatter};

pub struct User {
    id: u64,
    pub username: String,
    pub email: String,
    password: Option<String>,
}

impl User {
    pub fn new(id: u64, username: String, email: String) -> User {
        User { id, username, email, password: None }
    }

    pub fn set_password(&mut self, new_password: String) {
        self.password = Some(new_password)
    }

    pub fn is_protected(&self) -> bool { self.password != None }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "({} | {} | {})", self.id, self.username, self.email)
    }
}
