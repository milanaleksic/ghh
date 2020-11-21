pub struct Github {
    user_token: String,
}

impl Github {
    pub fn new(user_token: String) -> Self {
        Github { user_token }
    }
}