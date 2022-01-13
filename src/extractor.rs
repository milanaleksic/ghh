use std::collections::HashMap;

use crate::config::{Config, Repo};
use crate::github::Github;
use crate::refs::{LocalRefExtractor, Reference, RemoteRefExtractor};

pub struct Extractor {
    repos: Vec<Repo>,
    days: i32,
    remote_ref_extractor: RemoteRefExtractor,
    local_ref_extractor: LocalRefExtractor,
    github: Github,
}

impl Extractor {
    pub fn new(config: Config, days: i32) -> Self {
        Extractor {
            repos: config.repos.to_vec(),
            days,
            remote_ref_extractor: RemoteRefExtractor::new(),
            local_ref_extractor: LocalRefExtractor::new(),
            github: config.github(),
        }
    }
}

pub struct IssueDetails {
    pub issue_url: String,
    pub issue_title: String,
    pub messages: Vec<String>,
}

impl Extractor {
    pub fn extract(&self) -> HashMap<String, IssueDetails> {
        let mut map = HashMap::new();

        &self.repos
            .iter()
            .flat_map(|r| {
                let url = r.extract_repo_url();
                let commits = r.extract_commits(self.days);
                return commits
                    .lines()
                    .flat_map(|l| {
                        let mut list: Vec<Reference> = Vec::new();
                        list.append(&mut self.local_ref_extractor.extract(l, &url));
                        list.append(&mut self.remote_ref_extractor.extract(l));
                        list
                    })
                    .collect::<Vec<Reference>>();
            })
            .for_each(|r| {
                let entry: &mut IssueDetails = map.entry(r.full_issue_url.clone()).or_insert(IssueDetails {
                    issue_url: r.full_issue_url.clone(),
                    issue_title: String::new(),
                    messages: vec![],
                });
                if entry.issue_title == "" {
                    entry.issue_title = self.github.get_issue(r.full_issue_url.clone())
                        .map(|i|i.title)
                        .or_else(|| {
                            log::error!("Seems that reference {} is wrong in message \"{}\"", r.full_issue_url, r.message);
                            Some(String::from("???"))
                        })
                        .unwrap();
                }
                entry.messages.push(r.message);
            });

        map
    }
}
