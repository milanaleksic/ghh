use chrono::{DateTime, FixedOffset};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde_derive::Deserialize;

use crate::date_serializer;

#[derive(Deserialize, Debug)]
pub struct Assignee {
    pub login: String,
}

#[derive(Deserialize, Debug)]
pub struct Label {
    pub name: String,
    pub color: String,
}

#[derive(Deserialize, Debug)]
pub struct Milestone {
    pub title: String,
}

#[derive(Deserialize, Debug)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub assignees: Vec<Assignee>,
    pub body: String,
    pub labels: Vec<Label>,
    pub milestone: Option<Milestone>,
    pub closed_at: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Card {
    pub id: u64,
    #[serde(with = "date_serializer")]
    pub updated_at: DateTime<FixedOffset>,
    pub content_url: String,
}

#[derive(Deserialize, Debug)]
pub struct IssueSearchResult {
    pub items: Vec<Issue>,
}

pub struct Github {
    user_token: String,
}

impl Github {
    pub(crate) fn delete_card(&self, card_id: u64) {
        let request_url = format!("https://api.github.com/projects/columns/cards/{}", card_id);
        log::info!("Requesting GH DELETE URL: {}", request_url);
        let response = Client::new()
            .delete(&request_url)
            .basic_auth("", Some(self.user_token.clone()))
            .header("User-Agent", "ghh")// mandatory or we get 403!
            .header("Accept", "application/vnd.github.inertia-preview+json")
            .send()
            .unwrap();

        return match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => {
                log::info!("Card removed: {:?}", card_id);
            }
            s => {
                log::error!("Received response status: {:?}", s);
            }
        };
    }

    pub(crate) fn list_cards_on_board_column(&self, column_id: i32) -> Vec<Card> {
        let mut cards = Vec::new();
        let mut page = 1;
        let per_page = 100;

        loop {
            let request_url = format!("https://api.github.com/projects/columns/{}/cards", column_id);
            log::info!("Requesting GH GET URL: {}, page {}", request_url, page);
            let response = Client::new()
                .get(&request_url)
                .query(&[("page", page.to_string()), ("per_page", per_page.to_string())])
                .basic_auth("", Some(self.user_token.clone()))
                .header("User-Agent", "ghh")// mandatory or we get 403!
                .header("Accept", "application/vnd.github.inertia-preview+json")
                .send()
                .unwrap();
            match response.status() {
                StatusCode::OK => {
                    let mut page_cards: Vec<Card> = response.json().unwrap();
                    log::info!("received {} cards", page_cards.len());
                    cards.append(&mut page_cards);
                    if page_cards.len() < per_page {
                        break;
                    }
                    page += 1;
                }
                s => {
                    log::error!("Received response status: {:?}", s);
                    break
                }
            };
        }

        cards
    }

    pub(crate) fn get_owned_issue(&self, assignee: &String, repo: &String) -> Option<IssueSearchResult> {
        let request_url = "https://api.github.com/search/issues";
        log::info!("Requesting GH GET URL: {}", request_url);
        let response = Client::new()
            .get(request_url)
            .query(&[("q", format!("is:open is:issue assignee:{} repo:{}", assignee, repo))])
            .basic_auth("", Some(self.user_token.clone()))
            .header("User-Agent", "ghh")// mandatory or we get 403!
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .unwrap();

        return match response.status() {
            StatusCode::OK => {
                match response.json() {
                    Ok(issues)=> Some(issues),
                    Err(_) => {
                        log::error!("No content from issue {}, check if contents is set; can't continue further", &request_url);
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


    pub(crate) fn get_issue(&self, issue_url: String) -> Option<Issue> {
        let request_url = if issue_url.starts_with("https://api.github.com/repos") {
            issue_url
        } else {
            issue_url.replace("github.com", "api.github.com/repos")
        };
        log::info!("Requesting GH GET URL: {}", request_url);
        let response = Client::new()
            .get(&request_url)
            .basic_auth("", Some(self.user_token.clone()))
            .header("User-Agent", "ghh")// mandatory or we get 403!
            .send()
            .unwrap();

        return match response.status() {
            StatusCode::OK => {
                match response.json() {
                    Ok(issue)=> Some(issue),
                    Err(_) => {
                        log::error!("No content from issue {}, check if contents is set; can't continue further", &request_url);
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

impl Github {
    pub fn new(user_token: String) -> Self {
        Github { user_token }
    }
}