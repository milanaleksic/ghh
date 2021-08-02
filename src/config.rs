use std::fs::File;
use std::io::Read;
use std::process::{Command, Stdio};
use std::env;
use std::path::PathBuf;

use serde_derive::Deserialize;
use crate::github::Github;

#[derive(Deserialize)]
pub struct Config {
    #[serde(alias = "repo")]
    pub repos: Vec<Repo>,
    pub user_name: String,
    user_token: String,
}

impl Config {
    pub fn parse() -> Config {
        let path_buf = dirs::config_dir().unwrap().join("ghh").join("config.toml");
        let config_file_loc = path_buf.as_path();
        let config: Config = match File::open(config_file_loc) {
            Ok(mut file) => {
                let mut contents = Vec::new();
                file.read_to_end(&mut contents).unwrap();
                toml::from_slice(contents.as_slice()).unwrap()
            }
            Err(_) => {
                log::error!("Default config file not found: {}", config_file_loc.to_str().unwrap());
                // TODO: replace with Result response
                Config {
                    repos: vec![],
                    user_token: "".to_string(),
                    user_name: "".to_string(),
                }
            }
        };
        config
    }

    pub fn identify_active_repo(&self, repo: Option<String>) -> Result<Repo, String> {
        repo
            .clone()
            .map_or(env::current_dir(), |a| Ok(PathBuf::from(a)))
            .map_err(|err| err.to_string().clone())
            .and_then(|d| d.to_str()
                .ok_or(format!("failed to parse path {:?}", d).clone())
                .map(|s| s.to_string())
            )
            .and_then(|path| {
                self.repos.iter()
                    .filter(|r| r.location.starts_with(path.as_str()))
                    .map(|r| r.clone())
                    .next().ok_or(format!("No known (configured) repo matched {:?}", path).clone())
            })
    }

    pub fn github(&self) -> Github {
        return Github::new(self.user_token.clone())
    }
}

#[derive(Deserialize, Debug)]
pub struct Repo {
    location: String,
    author: String,
    pub in_progress_column: Option<i32>,
}

impl Clone for Repo {
    fn clone(&self) -> Self {
        return Repo {
            location: self.location.clone(),
            author: self.author.clone(),
            in_progress_column: self.in_progress_column.clone(),
        };
    }

    fn clone_from(&mut self, source: &Self) {
        self.location = source.location.clone();
        self.author = source.author.clone();
        self.in_progress_column = source.in_progress_column.clone();
    }
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
                self.author.as_str(),
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
