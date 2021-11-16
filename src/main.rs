use clap::Clap;
use log::LevelFilter::{Debug, Info};
use env_logger::Builder;

mod config;
mod refs;
mod extractor;
mod github;
mod date_serializer;
mod cmd_branch_from_issue;
mod cmd_task_cleanup;
mod cmd_daily;
mod cmd_epic_analysis;

// TODO: do not hardcode
const VERSION: &str = "1.0.4";

#[derive(Clap)]
#[clap(version = VERSION, author = "Milan Aleksić <milan@aleksic.dev>")]
struct Opts {
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,

    /// action to run
    #[clap(subcommand)]
    action: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Daily(cmd_daily::Daily),
    TaskCleanup(cmd_task_cleanup::TaskCleanup),
    BranchFromIssue(cmd_branch_from_issue::BranchFromIssue),
    EpicAnalysis(cmd_epic_analysis::EpicAnalysis),
}

fn main() {
    let command = Opts::parse();
    match command.verbose {
        0 => env_logger::init(),
        1 => Builder::new().filter(None, Info).init(),
        2 | _ => Builder::new().filter(None, Debug).init(),
    }
    match command.action {
        SubCommand::Daily(args) => args.run(),
        SubCommand::TaskCleanup(args) => args.run(),
        SubCommand::BranchFromIssue(args) => args.run(),
        SubCommand::EpicAnalysis(args) => args.run(),
    }
}
