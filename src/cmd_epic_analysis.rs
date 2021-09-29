use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry::Occupied;
use std::rc::Rc;

use clap::{ArgSettings, Clap};

use crate::config::Config;
use crate::github::{Github, Issue};
use crate::refs::{LocalRefExtractor, Reference};

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
    #[clap(long, setting = ArgSettings::MultipleOccurrences)]
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
    pub(crate) fn run(self) {
        Logic::new(self).run()
    }
}

struct Logic {
    cmd: Rc<EpicAnalysis>,
    node_titles: HashMap<u64, String>,
    cluster_members: HashMap<String, HashSet<u64>>,
    issue_graph: HashMap<u64, HashSet<u64>>,
    closed_issues: HashSet<u64>,
    issues_outside_milestone: HashSet<u64>,
}

impl Logic {
    pub fn new(cmd: EpicAnalysis) -> Self {
        Logic {
            cmd: Rc::new(cmd),
            node_titles: HashMap::new(),
            cluster_members: HashMap::new(),
            issue_graph: HashMap::new(),
            closed_issues: HashSet::new(),
            issues_outside_milestone: HashSet::new(),
        }
    }

    pub(crate) fn run(&mut self) {
        let config = Config::parse();
        let local_ref_extractor = LocalRefExtractor::new();
        let github = config.github();
        let repo_url = config.identify_active_repo(self.cmd.repo.clone()).unwrap().extract_repo();
        let epic_url = format!("{}/issues/{}", repo_url, self.cmd.epic_id);
        let epic_issue = github
            .get_issue(epic_url.clone())
            .ok_or(String::from(format!("Could not get issue {}", epic_url.clone())))
            .unwrap();
        let refs = local_ref_extractor.extract(epic_issue.body.as_str(), &repo_url);
        let internal_refs = refs.iter().map(|r| r.number).collect::<HashSet<u64>>();
        let issue_cache: RefCell<HashMap<String, Rc<Issue>>> = RefCell::new(HashMap::new());

        log::info!("References found in epic: {}", refs.len());

        for r in refs.iter() {
            let issue = self.cached_fetch_issue(&github, &issue_cache, r, false);
            let mut lines_with_deps = issue.body.lines()
                .filter(|l| l.starts_with(&self.cmd.prefix_blocked))
                .collect::<Vec<_>>();
            let mut encountered = false;
            for line in issue.body.lines() {
                if line.starts_with(&self.cmd.prefix_blocked) {
                    encountered = true;
                } else if encountered {
                    if line.starts_with("-") {
                        lines_with_deps.push(line);
                    } else {
                        break;
                    }
                }
            }
            let lines_with_deps = &lines_with_deps;
            for l in lines_with_deps {
                for fr in local_ref_extractor.extract(l, &repo_url) {
                    log::info!("Reference to a blocking issue found: {}->{}", fr.number.clone(), r.number.clone());
                    self.cached_fetch_issue(&github, &issue_cache, &fr, !internal_refs.contains(&fr.number));
                    self.issue_graph.entry(fr.number.clone())
                        .or_insert(HashSet::new())
                        .insert(r.number.clone());
                }
            }
            self.validate_blocked_label_state(&issue);
            self.validate_milestone(&epic_issue, &issue);
        }

        if self.cmd.graph {
            let unblocked_issues = self.find_unblocked_issues(internal_refs);
            self.build_graph(unblocked_issues);
        }
    }

    fn cached_fetch_issue(&mut self,
                          github: &Github,
                          issue_cache: &RefCell<HashMap<String, Rc<Issue>>>,
                          r: &Reference,
                          external: bool,
    ) -> Rc<Issue> {
        let issue: Rc<Issue> = {
            let mut ref_mut = issue_cache.borrow_mut();
            let e = ref_mut
                .entry(r.full_issue_url.clone());
            if let Occupied(_) = e {
                log::info!("Using from cache issue {}", r.full_issue_url);
            }
            e.or_insert_with(|| Rc::new(self.fetch_issue(&github, r, external)))
                .clone()
        };
        issue
    }

    fn fetch_issue(&mut self,
                   github: &Github,
                   r: &Reference,
                   from_outside: bool,
    ) -> Issue {
        let issue = github
            .get_issue(r.full_issue_url.clone())
            .unwrap();

        if from_outside {
            self.issues_outside_milestone.insert(issue.number);
        }

        if issue.closed_at.is_some() {
            self.closed_issues.insert(issue.number);
        }

        issue.labels.iter().for_each(|l| {
            if self.cmd.label_component.contains(&l.name) {
                self.cluster_members.entry(l.name.clone())
                    .or_insert(HashSet::new())
                    .insert(r.number.clone());
            }
        });

        self.node_titles.entry(issue.number.clone()).or_insert(issue.title.clone());
        issue
    }

    fn validate_blocked_label_state(&self, re: &Issue) {
        let has_blockages = self.issue_graph
            .iter()
            .filter(|sat| sat.1.contains(&re.number))
            .filter(|sat| !self.closed_issues.contains(&sat.0))
            .next()
            .is_some();
        if has_blockages {
            if !re.labels.iter().any(|l| l.name == self.cmd.label_blocked) {
                log::error!("Issue {} does not have blocked label '{}' but it blocks on something", re.number, self.cmd.label_blocked)
            }
        } else {
            if re.labels.iter().any(|l| l.name == self.cmd.label_blocked) {
                log::error!("Issue {} has blocked label '{}' but no upstream blockages found", re.number, self.cmd.label_blocked)
            }
        }
    }

    fn validate_milestone(&self, epic_issue: &Issue, issue: &Issue) {
        epic_issue.milestone.iter().for_each(|m| {
            let epic_issue_milestone = &m.title;
            if issue.milestone.is_none() {
                log::error!("Issue {} does not belong to the same milestone '{}' as the epic issue", issue.number, epic_issue_milestone);
                return;
            }
            issue.milestone.iter().for_each(|m_issue| {
                if &m_issue.title != epic_issue_milestone {
                    log::error!("Issue {} does not belong to the same milestone '{}' as the epic issue", issue.number, epic_issue_milestone);
                }
            })
        })
    }

    fn build_graph(&self, unblocked_issues: HashSet<u64>) {
        println!("digraph {{ ");
        self.issue_graph.iter().for_each(|g| {
            g.1.iter().for_each(|fr| {
                println!("{} -> {};", g.0, fr);
            });
        });
        self.cluster_members.iter().for_each(|c| {
            let cluster_name = format!("{}", c.0);
            println!("subgraph \"cluster{}\" {{", cluster_name);
            c.1.iter().for_each(|issue_id| {
                println!("{};", issue_id);
            });
            println!("label=\"{}\";", cluster_name);
            println!("}}");
        });
        self.node_titles.iter().for_each(|n| {
            self.node_titles.get(n.0).iter().for_each(|title| {
                let title = title
                    .replace("`", "")
                    .replace("<", "")
                    .replace(">", "");
                let len = title.len();
                let max1 = if len > 10 { 10 } else { len };
                let max2 = if len > 20 { 20 } else { len };
                let max3 = if len > 30 { 30 } else { len };
                let max4 = if len > 40 { 40 } else { len };
                let style = if self.closed_issues.contains(n.0) {
                    "style=filled,color=gray90,"
                } else if unblocked_issues.contains(n.0) {
                    ""
                } else {
                    "style=filled,color=indianred1,"
                };
                let external = if self.issues_outside_milestone.contains(n.0) {
                    "EXTERNAL<BR/> "
                } else {
                    ""
                };
                println!("{} [{}label=<{}issue #{}<BR />\
                <FONT POINT-SIZE='12' color='gray20'>{}<BR />{}<BR />{}<BR />{}</FONT>>]",
                         n.0,
                         style,
                         external,
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

    fn find_unblocked_issues(&self, internal_refs: HashSet<u64>) -> HashSet<u64> {
        let mut unblocked_issues = internal_refs.clone();
        self.issue_graph.iter().for_each(|g| {
            g.1.iter().for_each(|fr| {
                if !self.closed_issues.contains(g.0) {
                    unblocked_issues.remove(fr);
                }
            });
        });
        unblocked_issues
    }
}
