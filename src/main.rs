use std::env;

use config::Config;
use extractor::Extractor;
use crate::github::Github;
use chrono::Utc;

mod config;
mod refs;
mod extractor;
mod github;
mod date_serializer;

fn main() {
    // TODO: use GH API instead of git CLI
    // TODO: make references to the commits
    // TODO: remote ref matcher should say from which repo it comes for cross-ref
    let args: Vec<String> = env::args().collect();
    match args.get(1) {
        Some(mode) => match mode.as_ref() {
            "daily" => daily(&args[2..]),
            "task-cleanup" => task_cleanup(&args[2..]),
            _ => eprintln!("Please choose mode of working: daily, task-cleanup")
        }
        None => eprintln!("Please choose mode of working: daily, task-cleanup")
    }
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
            eprintln!("Card with id {} updated recently at {}", c.id, c.updated_at)
        } else {
            eprintln!("Deleting old card with id {} last updated at {}", c.id, c.updated_at);
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
