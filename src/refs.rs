use regex::Regex;

pub struct Reference {
    pub full_issue_url: String,
    pub message: String,
    pub number: u64,
}

pub struct LocalRefExtractor {
    local_ref_regex: Regex,
}

impl LocalRefExtractor {
    pub fn new() -> Self {
        LocalRefExtractor { local_ref_regex: Regex::new(r" #([0-9]*)").unwrap() }
    }

    pub fn extract(&self, l: &str, url: &String) -> Vec<Reference> {
        let local_msg = self.local_ref_regex.replace(l, "").to_string().clone();
        self.local_ref_regex
            .captures_iter(l)
            .map(|m| {
                let issue = m.get(1).unwrap();
                Reference {
                    full_issue_url: format!("{}/issues/{}", url, issue.as_str()),
                    message: local_msg.clone(),
                    number: issue.as_str().parse().unwrap(),
                }
            })
            .collect::<Vec<Reference>>()
    }
}

pub struct RemoteRefExtractor {
    remote_ref_regex: Regex,
}

impl RemoteRefExtractor {
    pub fn new() -> Self {
        RemoteRefExtractor { remote_ref_regex: Regex::new(r" ([^/]+/[^#]+)#([0-9]*)").unwrap() }
    }

    pub fn extract(&self, l: &str) -> Vec<Reference> {
        let remote_msg = self.remote_ref_regex.replace(l, "").to_string().clone();
        self.remote_ref_regex
            .captures_iter(l)
            .map(|m| {
                let issue = m.get(2).unwrap();
                Reference {
                    full_issue_url: format!("https://github.com/{}/issues/{}",
                                            m.get(1).unwrap().as_str(),
                                            issue.as_str()),
                    message: remote_msg.clone(),
                    number: issue.as_str().parse().unwrap(),
                }
            })
            .collect::<Vec<Reference>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_ref() {
        let vec1 = RemoteRefExtractor::new().extract("ref org/repo#1375: some issue");
        assert_eq!(
            vec1.get(0).unwrap().message,
            "ref: some issue".to_string(),
        );
        assert_eq!(
            vec1.get(0).unwrap().full_issue_url,
            "https://github.com/org/repo/issues/1375".to_string(),
        );
    }
}