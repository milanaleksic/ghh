use clap::Clap;
use regex::Regex;

use crate::config::Config;
use crate::github::Github;

/// Propose branch name based on actively assigned project cards in "In Progress" column
#[derive(Clap)]
pub(crate) struct BranchFromIssue {
    /// Which repo location should be parsed
    #[clap(short)]
    repo: Option<String>,
}

impl BranchFromIssue {
    pub(crate) fn run(&self) {
        let config = Config::parse();
        let github = Github::new(config.user_token.clone());
        let repo = config.identify_active_repo(self.repo.clone());
        let author = &config.user_name.clone();
        repo.and_then(|r| {
            r.in_progress_column.ok_or(format!(
                "repo {} doesn't have in_progress_column set",
                &r.location
            ))
        })
        .map(|column_id| github.list_cards_on_board_column(column_id))
        .map(|cards| {
            cards
                .iter()
                .filter_map(|c| github.get_issue(c.content_url.clone()))
                .filter(|i| {
                    i.assignees
                        .iter()
                        .map(|a| a.login.clone())
                        .any(|l| l.eq(author.as_str()))
                })
                .for_each(|c| println!("{}_{}", c.number, BranchFromIssue::stupify(c.title)))
        })
        .map_err(|err| log::error!("Error: {}", err))
        .ok();
    }

    fn stupify(title: String) -> String {
        Regex::new(r"[\W]+")
            .unwrap()
            .replace_all(title.to_lowercase().as_str(), "_")
            .to_string()
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
