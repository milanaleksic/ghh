use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Issue {
    pub key: String,
    pub fields: IssueFields,
}

#[derive(Deserialize, Debug)]
pub struct IssueFields {
    pub summary: String,
}

#[derive(Deserialize, Debug)]
pub struct JiraIssueSearchResult {
    pub issues: Vec<Issue>,
}

pub struct Jira {
    username: String,
    api_token: String,
    jira_url: String,
}

impl Jira {

    pub(crate) fn get_owned_issues(&self) -> Option<JiraIssueSearchResult> {
        let request_url = format!("{}/rest/api/2/search", self.jira_url);
        log::info!("Requesting JIRA GET URL: {}", request_url);
        let response = Client::new()
            .get(&request_url)
            .query(&[("jql", "status=\"In Progress\" AND assignee=currentUser()")])
            .basic_auth(self.username.clone(), Some(self.api_token.clone()))
            .header("User-Agent", "ghh")
            .send()
            .unwrap();

        return match response.status() {
            StatusCode::OK => {
                match response.json() {
                    Ok(issue_search_result)=> Some(issue_search_result),
                    Err(x) => {
                        log::error!("No content from issue {}, check if contents is set; can't continue further (details: {:?})", &request_url, x);
                        None
                    }
                }
            }
            s => {
                log::error!("Received response status: {:?}", s);
                None
            }
        };
    }

}

impl Jira {
    pub fn new(username: String, jira_url: String, api_token: String) -> Self {
        Jira { username, jira_url, api_token }
    }
}