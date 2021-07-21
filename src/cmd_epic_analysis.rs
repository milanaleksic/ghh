use clap::Clap;

use crate::config::Config;
use crate::github::Github;
use crate::refs::{LocalRefExtractor};

/// Remove old project cards by archiving them
#[derive(Clap)]
pub(crate) struct EpicAnalysis {
    /// What is the epic ID?
    #[clap(short, long)]
    epic_id: i32,

    /// Which repo location should be parsed
    #[clap(short, long)]
    repo: Option<String>,
}

impl EpicAnalysis {
    pub(crate) fn run(&self) {
        let config = Config::parse();
        let local_ref_extractor = LocalRefExtractor::new();
        let github = Github::new(config.user_token.clone());
        config.identify_active_repo(self.repo.clone())
            .and_then(|r| {
                let repo_url = r.extract_repo();
                github
                    .get_issue(format!("{}/issues/{}", repo_url, self.epic_id))
                    .ok_or(String::from("Could not get issue"))
                    .and_then(|e| {
                        let refs = local_ref_extractor.extract(e.body.as_str(), &repo_url);
                        print!("References found in epic: {}", refs.len());
                        Ok(refs)
                    })
            })
            .unwrap();
        ()
        // let issue_content = github.get_issue(issue_url: String)
        // let cards = github.list_cards_on_board_column(self.column_id);
        // cards.iter().for_each(|c| {
        //     let since = Utc::now().signed_duration_since(c.updated_at).num_days() as i32;
        //     if since < self.days {
        //         log::info!("Card with id {} updated recently at {}", c.id, c.updated_at)
        //     } else {
        //         log::warn!("Deleting old card with id {} last updated at {}", c.id, c.updated_at);
        //         github.delete_card(c.id)
        //     }
        // })
    }
}
