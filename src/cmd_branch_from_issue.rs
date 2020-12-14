use std::env;
use std::path::PathBuf;

use clap::Clap;
use regex::Regex;

use crate::config::{Config, Repo};
use crate::github::Github;

/// Propose branch name based on actively assigned project cars in "done" column
#[derive(Clap)]
pub(crate) struct BranchFromIssue {
    /// Which repo location should be parsed
    #[clap(short)]
    repo: Option<String>
}

impl BranchFromIssue {
    pub(crate) fn run(&self) {
        let config = Config::parse();
        let github = Github::new(config.user_token.clone());
        let repo = self.identify_active_repo(&config);
        let author = &config.user_name.clone();
        repo
            .and_then(|r| {
                r.in_progress_column.ok_or(format!("repo {} doesn't have in_progress_column set", &r.location))
            })
            .map(|column_id| github.list_cards_on_board_column(column_id))
            .map(|cards|
                cards
                    .iter()
                    .filter_map(|c| github.get_issue(c.content_url.clone()))
                    .filter(|i| {
                        i.assignees.iter()
                            .map(|a| a.login.clone())
                            .any(|l| l.eq(author.as_str()))
                    })
                    .for_each(|c| {
                        println!("{}_{}", c.number, BranchFromIssue::stupify(c.title))
                    })
            )
            .map_err(|err| log::error!("Error: {}", err))
            .ok();
    }

    fn identify_active_repo(&self, config: &Config) -> Result<Repo, String> {
        self
            .repo
            .clone()
            .map_or(env::current_dir(), |a| Ok(PathBuf::from(a)))
            .map_err(|err| err.to_string().clone())
            .and_then(|d| d.to_str()
                .ok_or(format!("failed to parse path {:?}", d).clone())
                .map(|s| s.to_string())
            )
            .and_then(|path| {
                config.repos.iter()
                    .filter(|r| r.location.starts_with(path.as_str()))
                    .map(|r| r.clone())
                    .next().ok_or(format!("No known (configured) repo matched {:?}", path).clone())
            })
    }

    fn stupify(title: String) -> String {
        Regex::new(r"[\W]+").unwrap().replace_all(title.to_lowercase().as_str(), "_").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_replace() {
        assert_eq!(
            BranchFromIssue::stupify("1412_Extend TestResult` model for ChangeOverTime evaluations".to_string()),
            "1412_extend_testresult_model_for_changeovertime_evaluations"
        )
    }
}