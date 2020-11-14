use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::{Command, Stdio};

use regex::Regex;
use serde_derive::Deserialize;

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

struct LocalRefExtractor {
    local_ref_regex: Regex,
}

impl LocalRefExtractor {
    pub fn new() -> Self {
        LocalRefExtractor { local_ref_regex: Regex::new(r" #([0-9]*)").unwrap() }
    }

    fn extract(&self, l: &str, url: &String) -> Vec<(String, String)> {
        let local_msg = self.local_ref_regex.replace(l, "").to_string().clone();
        self.local_ref_regex
            .captures_iter(l)
            .map(|m| (format!("{}/issues/{}", url, m.get(1).unwrap().as_str()), local_msg.clone()))
            .collect::<Vec<(String, String)>>()
    }
}

struct RemoteRefExtractor {
    remote_ref_regex: Regex,
}

impl RemoteRefExtractor {
    pub fn new() -> Self {
        RemoteRefExtractor { remote_ref_regex: Regex::new(r" (github.com/.*)#([0-9]*)").unwrap() }
    }

    fn extract(&self, l: &str) -> Vec<(String, String)> {
        let remote_msg = self.remote_ref_regex.replace(l, "").to_string().clone();
        self.remote_ref_regex
            .captures_iter(l)
            .map(|m| (format!("https://{}/issues/{}", m.get(1).unwrap().as_str(), m.get(2).unwrap().as_str()), remote_msg.clone()))
            .collect::<Vec<(String, String)>>()
    }
}

struct Extractor {
    config: Config,
    days: i32,
    remote_ref_extractor: RemoteRefExtractor,
    local_ref_extractor: LocalRefExtractor,
}

impl Extractor {
    pub fn new(config: Config, days: i32) -> Self {
        Extractor {
            config,
            days,
            remote_ref_extractor: RemoteRefExtractor::new(),
            local_ref_extractor: LocalRefExtractor::new(),
        }
    }
}

impl Extractor {
    pub fn extract(&self) -> HashMap::<String, Vec<String>> {
        let mut map = HashMap::new();

        &self.config.repo
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

fn main() {
    // TODO: use GH API
    let args: Vec<String> = env::args().collect();
    let days = args
        .get(1)
        .map_or(1, |a| a.parse().unwrap());

    let path_buf = dirs::config_dir().unwrap().join("ghh").join("config.toml");
    let config_file_loc = path_buf.as_path();
    let config: Config = match File::open(config_file_loc) {
        Ok(mut file) => {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).unwrap();
            toml::from_slice(contents.as_slice()).unwrap()
        }
        Err(_) => {
            eprintln!("Default config file not found: {}", config_file_loc.to_str().unwrap());
            Config{ repo: vec![] }
        }
    };

    let extractor = Extractor::new(config, days);
    let map = extractor.extract();

    let mut issues = map.keys().map(|k| k.clone()).collect::<Vec<String>>();
    issues.sort();
    for issue in &issues {
        println!("{}", issue);
        map.get(issue).unwrap().iter().for_each(|msg| println!("• {}", msg));
        println!("\n");
    }
}
