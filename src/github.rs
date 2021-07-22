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
}

#[derive(Deserialize, Debug, Clone)]
pub struct Card {
    pub id: u64,
    #[serde(with = "date_serializer")]
    pub updated_at: DateTime<FixedOffset>,
    pub content_url: String,
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

        loop {
            let request_url = format!("https://api.github.com/projects/columns/{}/cards", column_id);
            log::info!("Requesting GH GET URL: {}, page {}", request_url, page);
            let response = Client::new()
                .get(&request_url)
                .query(&[("page", page.to_string())])
                .basic_auth("", Some(self.user_token.clone()))
                .header("User-Agent", "ghh")// mandatory or we get 403!
                .header("Accept", "application/vnd.github.inertia-preview+json")
                .send()
                .unwrap();
            match response.status() {
                StatusCode::OK => {
                    let mut page_cards: Vec<Card> = response.json().unwrap();
                    if page_cards.is_empty() {
                        break;
                    }
                    cards.append(&mut page_cards);
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
                let issue: Issue = response.json().unwrap();
                Some(issue)
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