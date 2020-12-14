use chrono::Utc;
use clap::Clap;

use crate::config::Config;
use crate::github::Github;

/// Remove old project cards by archiving them
#[derive(Clap)]
pub(crate) struct TaskCleanup {
    /// what is the column id to cleanup?
    #[clap(short, long, default_value = "1")]
    column_id: i32,

    /// archive cards older than <days> days
    #[clap(short, long, default_value = "7")]
    days: i32,
}

impl TaskCleanup {
    pub(crate) fn run(&self) {
        let config = Config::parse();
        let github = Github::new(config.user_token.clone());
        let cards = github.list_cards_on_board_column(self.column_id);
        cards.iter().for_each(|c| {
            let since = Utc::now().signed_duration_since(c.updated_at).num_days() as i32;
            if since < self.days {
                log::info!("Card with id {} updated recently at {}", c.id, c.updated_at)
            } else {
                log::warn!("Deleting old card with id {} last updated at {}", c.id, c.updated_at);
                github.delete_card(c.id)
            }
        })
    }
}