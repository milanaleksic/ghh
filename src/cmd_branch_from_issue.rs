use std::collections::HashSet;

use clap::Parser;
use regex::Regex;

use crate::config::{Config, Repo};
use crate::github::Issue;

/// Propose branch name based on actively assigned project cards in "In Progress" column
#[derive(Parser)]
pub(crate) struct BranchFromIssue {
    /// Which repo location should be parsed
    #[clap(short)]
    repo: Option<String>,

    /// if set, only active issues will be shown (the ones present as cards in in_progress_column)
    #[clap(long)]
    only_active_column: bool,
}

impl BranchFromIssue {
    pub(crate) fn run(&self) {
        let config = Config::parse();
        let repo_config = self.get_repo(&config);
        if repo_config.is_err() {
            log::error!("Repository is not identified: {}", repo_config.err().unwrap());
            return;
        }
        let repo = repo_config.unwrap();
        if let Some(true) = repo.uses_jira {
            self.process_jira(config)
        } else {
            self.process_gh(config, repo)
        }
    }

    fn get_repo(&self, config: &Config) -> Result<Repo, String> {
        config.identify_active_repo(self.repo.clone())
    }

    fn stupify(title: String) -> String {
        Regex::new(r"[\W]+")
            .unwrap()
            .replace_all(title.to_lowercase().as_str(), "_")
            .to_string()
    }

    fn process_jira(&self, config: Config) {
        let jira = config.jira();
        let my_issues = jira.get_owned_issues().unwrap();
        log::info!("Got {} owned issues", my_issues.issues.len());
        my_issues.issues.iter()
            .for_each(|c| println!("{}_{}", c.key, BranchFromIssue::stupify(c.fields.summary.clone())));
    }

    fn process_gh(&self, config: Config, repo: Repo) {
        let github = config.github();
        let author = &config.user_name.clone();
        let owned_issues = github
            .get_owned_issue(&config.user_name.clone(), &repo.extract_repo_name())
            .ok_or(String::from("Repo not identified"));
        if owned_issues.is_err() {
            log::info!("No owned issues found");
            return;
        }
        let my_issues = owned_issues.unwrap().items;
        log::info!("Got {} owned issues", my_issues.len());
        let issues = if self.only_active_column {
            let interesting_issues: HashSet<u64> = my_issues.into_iter()
                .map(|i| i.number)
                .collect();
            // TODO: avoid another copy of repo here
            self.get_repo(&config)
                .and_then(|r| {
                    r.in_progress_column.ok_or(format!(
                        "repo {:?} doesn't have in_progress_column set",
                        r
                    ))
                })
                .map(|column_id| {
                    github.list_cards_on_board_column(column_id)
                        .iter()
                        .filter(|c| {
                            let issue_number = c.content_url.split("/").last().unwrap();
                            interesting_issues.contains(&issue_number.parse::<u64>().unwrap())
                        })
                        .filter_map(|c| github.get_issue(c.content_url.clone()))
                        .collect::<Vec<Issue>>()
                })
                .map_err(|err| log::error!("Error: {}", err))
                .ok()
                .unwrap()
        } else {
            my_issues
        };
        issues.iter()
            .filter(|i| {
                i.assignees
                    .iter()
                    .map(|a| a.login.clone())
                    .any(|l| l.eq(author.as_str()))
            })
            .for_each(|c| println!("{}_{}", c.number, BranchFromIssue::stupify(c.title.clone())));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_replace() {
        assert_eq!(
            BranchFromIssue::stupify(
                "1412_Extend TestResult` model for ChangeOverTime evaluations".to_string()
            ),
            "1412_extend_testresult_model_for_changeovertime_evaluations"
        )
    }
}
