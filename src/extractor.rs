use std::collections::HashMap;

use crate::config::{Config, Repo};
use crate::refs::{LocalRefExtractor, RemoteRefExtractor};
use crate::github::Github;

pub struct Extractor {
    repos: Vec<Repo>,
    days: i32,
    remote_ref_extractor: RemoteRefExtractor,
    local_ref_extractor: LocalRefExtractor,
    github: Github,
}

impl Extractor {
    pub fn new(config: Config, days: i32, github: Github) -> Self {
        Extractor {
            repos: config.repos.to_vec(),
            days,
            remote_ref_extractor: RemoteRefExtractor::new(),
            local_ref_extractor: LocalRefExtractor::new(),
            github,
        }
    }
}

impl Extractor {
    pub fn extract(&self) -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();

        &self.repos
            .iter()
            .flat_map(|r| {
                let url = r.extract_repo();
                let commits = r.extract_commits(self.days);
                return commits
                    .lines()
                    .flat_map(|l| {
                        let mut list: Vec<(String, String)> = Vec::new();
                        list.append(&mut self.local_ref_extractor.extract(l, &url));
                        list.append(&mut self.remote_ref_extractor.extract(l));
                        list
                    })
                    .collect::<Vec<(String, String)>>();
            })
            .for_each(|commit| {
                let entry: &mut Vec<String> = map.entry(commit.0).or_insert(Vec::new());
                entry.push(commit.1);
            });

        map
    }
}
