use std::env;

use config::Config;
use extractor::Extractor;
use crate::github::Github;

mod config;
mod refs;
mod extractor;
mod github;

fn main() {
    // TODO: use GH API instead of git CLI
    // TODO: make references to the commits
    // TODO: remote ref matcher should say from which repo it comes for cross-ref
    // TODO: use GH API to extract issue names
    let args: Vec<String> = env::args().collect();
    let days = args
        .get(1)
        .map_or(1, |a| a.parse().unwrap());

    let config = Config::parse();
    let github = Github::new(config.user_token.clone());
    let extractor = Extractor::new(config, days, github);
    let issue_url_to_messages = extractor.extract();

    let mut issues = issue_url_to_messages.keys()
        .map(|k| k.clone())
        .collect::<Vec<String>>();
    issues.sort();
    for issue in &issues {
        println!("{}", issue);
        issue_url_to_messages.get(issue).unwrap()
            .iter()
            .for_each(|msg| println!("• {}", msg));
        println!();
    }
}
