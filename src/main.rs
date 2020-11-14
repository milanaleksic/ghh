use std::process::{Command, Stdio};

use regex::Regex;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::env;

#[derive(Deserialize)]
struct Config {
    repo: Vec<Repo>,
}

#[derive(Deserialize)]
struct Repo {
    location: String,
}

impl Repo {
    pub fn extract_repo(&self) -> String {
        let output = Command::new("git")
            .current_dir(&self.location)
            .args("remote get-url --push origin".split(" "))
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start echo process")
            .wait_with_output()
            .expect("Failed to extract the origin URL");
        let remote_url = output
            .stdout
            .to_vec();
        return String::from_utf8(remote_url)
            .expect("Wrong output encoding")
            .replace("git@github.com:", "https://github.com/")
            .replace(".git\n", "");
    }

    pub fn extract_commits(&self, days: i32) -> String {
        let output2 = Command::new("git")
            .current_dir(&self.location)
            .args(vec![
                "log",
                format!("--since='{} days ago'", days).as_str(),
                "--oneline",
                "--pretty=format:%s",
                "--abbrev-commit",
                "--author",
                "Milan"
            ])
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start echo process")
            .wait_with_output()
            .expect("Failed to extract the origin URL");
        let commits_vec = output2
            .stdout
            .to_vec();
        return String::from_utf8(commits_vec)
            .expect("Wrong output encoding");
    }
}

fn main() {
    // TODO: use config file
    // TODO: use GH API
    let args: Vec<String> = env::args().collect();
    let days = args
        .get(1)
        .map_or(1, |a| a.parse().unwrap());
    let config: Config = toml::from_str(
        r#"
        [[repo]]
        location = '/Users/milan/SourceCode/soda'

        [[repo]]
        location = '/Users/milan/SourceCode/docs'

        [[repo]]
        location = '/Users/milan/SourceCode/terraform'
    "#,
    )
        .unwrap();

    let mut map = HashMap::new();
    let local_ref_regex = Regex::new(r" #([0-9]*)").unwrap();
    let remote_ref_regex = Regex::new(r" (github.com/.*)#([0-9]*)").unwrap();

    &config.repo
        .iter()
        .flat_map(|r| {
            let url = r.extract_repo();
            let commits = r.extract_commits(days);
            return commits
                .lines()
                .flat_map(|l| {
                    let local_msg = local_ref_regex.replace(l, "").to_string().clone();
                    let remote_msg = remote_ref_regex.replace(l, "").to_string().clone();
                    let mut list:Vec<(String, String)> = Vec::new();
                    list.append(&mut local_ref_regex
                        .captures_iter(l)
                        .map(|m| (format!("{}/issues/{}", url, m.get(1).unwrap().as_str()), local_msg.clone()))
                        .collect::<Vec<(String, String)>>());
                    list.append(&mut remote_ref_regex
                        .captures_iter(l)
                        .map(|m| (format!("https://{}/issues/{}", m.get(1).unwrap().as_str(), m.get(2).unwrap().as_str()), remote_msg.clone()))
                        .collect::<Vec<(String, String)>>());
                    list
                })
                .collect::<Vec<(String, String)>>();
        })
        .for_each(|commit| {
            let entry: &mut Vec<String> = map.entry(commit.0).or_insert(Vec::new());
            entry.push(commit.1);
        });

    let mut issues = map.keys().map(|k| k.clone()).collect::<Vec<String>>();
    issues.sort();
    for issue in &issues {
        println!("{}", issue);
        map.get(issue).unwrap().iter().for_each(|msg| println!("• {}", msg));
        println!("\n");
    }
}
