use std::borrow::Borrow;
use std::env;
use std::path::PathBuf;

use chrono::Utc;
use regex::Regex;

use config::Config;
use extractor::Extractor;

use crate::config::Repo;
use crate::github::Github;

mod config;
mod refs;
mod extractor;
mod github;
mod date_serializer;

fn main() {
    // TODO: use GH API instead of git CLI
    // TODO: make references to the commits
    // TODO: remote ref matcher should say from which repo it comes for cross-ref
    // TODO: cli args library for auto-help etc
    let args: Vec<String> = env::args().collect();
    match args.get(1) {
        Some(mode) => match mode.as_ref() {
            "daily" => daily(&args[2..]),
            "task-cleanup" => task_cleanup(&args[2..]),
            "branch-from-issue" => branch_from_issue(&args[2..]),
            _ => log::error!("Please choose mode of working: daily, task-cleanup, branch-from-issue"),
        }
        None => log::error!("Please choose mode of working: daily, task-cleanup, branch-from-issue"),
    }
}

fn branch_from_issue(args: &[String]) {
    let config = Config::parse();
    let github = Github::new(config.user_token.clone());
    let repo = identify_active_repo(config, args);
    let author = repo.borrow().as_ref().ok().map(|a| a.author.clone());
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
                        .any(|l| l.eq(author.as_ref().unwrap().as_str()))
                })
                .for_each(|c| {
                    println!("{}_{}", c.number, stupify(c.title))
                })
        )
        .map_err(|err| log::error!("Error: {}", err))
        .ok();
}

fn stupify(title: String) -> String {
    Regex::new(r"[\W]+").unwrap().replace_all(title.to_lowercase().as_str(), "_").to_string()
}

fn identify_active_repo(config: Config, args: &[String]) -> Result<Repo, String> {
    args
        .get(0)
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

fn task_cleanup(args: &[String]) {
    let column_id = args
        .get(0)
        .map_or(1, |a| a.parse().unwrap());
    let days = args
        .get(1)
        .map_or(7, |a| a.parse().unwrap());
    let config = Config::parse();
    let github = Github::new(config.user_token.clone());
    let cards = github.list_cards_on_board_column(column_id);
    cards.iter().for_each(|c| {
        if Utc::now().signed_duration_since(c.updated_at).num_days() < days {
            log::info!("Card with id {} updated recently at {}", c.id, c.updated_at)
        } else {
            log::warn!("Deleting old card with id {} last updated at {}", c.id, c.updated_at);
            github.delete_card(c.id)
        }
    })
}

fn daily(args: &[String]) {
    let days = args
        .get(0)
        .map_or(1, |a| a.parse().unwrap());

    let config = Config::parse();
    let github = Github::new(config.user_token.clone());
    let extractor = Extractor::new(config, days, github);
    let issue_url_to_messages = extractor.extract();

    let mut issues = issue_url_to_messages.keys()
        .map(|k| k.clone())
        .collect::<Vec<String>>();
    issues.sort();
    for issue_url in &issues {
        let issue_details = issue_url_to_messages.get(issue_url).unwrap();
        println!("*{}* ({})", issue_details.issue_title, issue_url);
        issue_details
            .messages
            .iter()
            .for_each(|msg| println!("• {}", msg));
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_replace() {
        assert_eq!(
            stupify("1412_Extend TestResult` model for ChangeOverTime evaluations".to_string()),
            "1412_extend_testresult_model_for_changeovertime_evaluations"
        )
    }
}