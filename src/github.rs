use reqwest::blocking::Client;
use serde_derive::Deserialize;
use reqwest::StatusCode;

pub struct Github {
    user_token: String,
}

#[derive(Deserialize, Debug)]
struct Issue {
    id: u64,
    title: String,
}

impl Github {
    pub(crate) fn get_issue_name(&self, issue_url: String) -> String {
        let request_url = issue_url.replace("github.com", "api.github.com/repos");
        eprintln!("Requesting GH URL: {}", request_url);
        let response = Client::new()
            .get(&request_url)
            .basic_auth("", Some(self.user_token.clone()))
            .header("User-Agent", "ghh")// mandatory or we get 403!
            .send()
            .unwrap();

        return match response.status() {
            StatusCode::OK => {
                let issue: Issue = response.json().unwrap();
                issue.title
            }
            s => {
                eprintln!("Received response status: {:?}", s);
                String::new()
            },
        };
    }
}

impl Github {
    pub fn new(user_token: String) -> Self {
        Github { user_token }
    }
}