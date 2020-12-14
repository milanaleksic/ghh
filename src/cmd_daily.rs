use clap::Clap;

use crate::config::Config;
use crate::extractor::Extractor;
use crate::github::Github;

/// Give run-down of all things done in the commits during the previous <days>
#[derive(Clap)]
pub(crate) struct Daily {
    /// how many days to go into history
    #[clap(short, long, default_value = "1")]
    days: i32,
}

impl Daily {
    pub(crate) fn run(&self) {
        let config = Config::parse();
        let github = Github::new(config.user_token.clone());
        let extractor = Extractor::new(config, self.days, github);
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
}