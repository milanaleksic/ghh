use std::collections::{HashMap, HashSet};

use clap::Clap;

use crate::config::Config;
use crate::github::{Github, Issue};
use crate::refs::LocalRefExtractor;

/// Remove old project cards by archiving them
#[derive(Clap)]
pub(crate) struct EpicAnalysis {
    /// What is the epic ID?
    #[clap(short, long)]
    epic_id: i32,

    /// Which repo location should be parsed
    #[clap(short, long)]
    repo: Option<String>,

    /// Which labels should be processed as "components" and thus trigger subgraph grouping
    #[clap(long)]
    label_component: Vec<String>,

    /// Which label should be used as "blocked" indicator
    #[clap(long, default_value = "blocked")]
    label_blocked: String,

    /// Which text is expected on the beginning of the line to demarcate list of upstream blockers
    #[clap(long, default_value = "Blocked by")]
    prefix_blocked: String,

    /// if active, GraphViz diagram code will be placed in stdout
    #[clap(long)]
    graph: bool,

}

impl EpicAnalysis {
    pub(crate) fn run(&self) {
        let config = Config::parse();
        let local_ref_extractor = LocalRefExtractor::new();
        let github = Github::new(config.user_token.clone());
        let repo = config.identify_active_repo(self.repo.clone()).unwrap();

        let repo_url = repo.extract_repo();
        let epic_url = format!("{}/issues/{}", repo_url, self.epic_id);
        let epic_issue = github
            .get_issue(epic_url.clone())
            .ok_or(String::from(format!("Could not get issue {}", epic_url.clone())))
            .unwrap();
        let refs = local_ref_extractor.extract(epic_issue.body.as_str(), &repo_url);
        log::info!("References found in epic: {}", refs.len());

        let mut node_titles = HashMap::new();
        let mut cluster_members = HashMap::new();
        let mut issue_graph = HashMap::new();
        refs.iter().for_each(|r| {
            let issue = github
                .get_issue(r.full_issue_url.clone())
                .ok_or(String::from(format!("Could not get issue {}", epic_url.clone())))
                .unwrap();

            issue.labels.iter().for_each(|l| {
                if self.label_component.contains(&l.name) {
                    cluster_members.entry(l.name.clone())
                        .or_insert(HashSet::new())
                        .insert(r.number.clone());
                }
            });

            node_titles.entry(issue.number.clone()).or_insert(issue.title.clone());
            let blocked_lines = issue.body.lines().filter(|l| l.starts_with(&self.prefix_blocked)).collect::<Vec<_>>();
            self.validate_blocked_label_state(&issue, &blocked_lines);
            blocked_lines.iter().for_each(|l| {
                local_ref_extractor.extract(l, &repo_url)
                    .iter()
                    .for_each(|fr| {
                        issue_graph.entry(fr.number.clone())
                            .or_insert(HashSet::new())
                            .insert(r.number.clone());
                    });
            });
            self.validate_milestone(&epic_issue, &issue);
        });

        if self.graph {
            self.build_graph(node_titles, cluster_members, issue_graph);
        }
    }

    fn validate_blocked_label_state(&self, re: &Issue, blocked_lines: &Vec<&str>) {
        if !blocked_lines.is_empty() {
            if !re.labels.iter().any(|l| l.name == self.label_blocked) {
                log::error!("Issue {} does not have blocked label '{}' but it blocks on something", re.number, self.label_blocked)
            }
        } else {
            if re.labels.iter().any(|l| l.name == self.label_blocked) {
                log::error!("Issue {} has blocked label '{}' but no upstream blockages found", re.number, self.label_blocked)
            }
        }
    }

    fn validate_milestone(&self, epic_issue: &Issue, issue: &Issue) {
        epic_issue.milestone.iter().for_each(|m| {
            let epic_issue_milestone = &m.title;
            if issue.milestone.is_none() {
                log::error!("Issue {} does not belong to the same milestone '{}' as the epic issue", issue.number, epic_issue_milestone);
                return
            }
            issue.milestone.iter().for_each(|m_issue| {
                if &m_issue.title != epic_issue_milestone {
                    log::error!("Issue {} does not belong to the same milestone '{}' as the epic issue", issue.number, epic_issue_milestone);
                }
            })
        })
    }

    fn build_graph(&self,
                   node_titles: HashMap<u64, String>,
                   cluster_members: HashMap<String, HashSet<u64>>,
                   issue_graph: HashMap<u64, HashSet<u64>>
    ) {
        println!("digraph {{ ");
        issue_graph.iter().for_each(|g| {
            g.1.iter().for_each(|fr| {
                println!("{} -> {};", g.0, fr);
            });
        });
        cluster_members.iter().for_each(|c| {
            let cluster_name = format!("{}", c.0);
            println!("subgraph \"cluster{}\" {{", cluster_name);
            c.1.iter().for_each(|issue_id| {
                println!("{};", issue_id);
            });
            println!("label=\"{}\";", cluster_name);
            println!("}}");
        });
        node_titles.iter().for_each(|n| {
            node_titles.get(n.0).iter().for_each(|title| {
                let title = title.replace("`", "");
                let len = title.len();
                let max1 = if len > 10 { 10 } else { len };
                let max2 = if len > 20 { 20 } else { len };
                let max3 = if len > 30 { 30 } else { len };
                let max4 = if len > 40 { 40 } else { len };
                println!("{} [label=<issue #{}<BR /><FONT POINT-SIZE='12' color='blue'>{}<BR />{}<BR />{}<BR />{}</FONT>>]",
                         n.0,
                         n.0,
                         title[0..max1].to_string(),
                         title[max1..max2].to_string(),
                         title[max2..max3].to_string(),
                         title[max3..max4].to_string(),
                )
            })
        });
        println!("}} ");
    }
}
