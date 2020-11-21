use regex::Regex;

pub struct LocalRefExtractor {
    local_ref_regex: Regex,
}

impl LocalRefExtractor {
    pub fn new() -> Self {
        LocalRefExtractor { local_ref_regex: Regex::new(r" #([0-9]*)").unwrap() }
    }

    pub fn extract(&self, l: &str, url: &String) -> Vec<(String, String)> {
        let local_msg = self.local_ref_regex.replace(l, "").to_string().clone();
        self.local_ref_regex
            .captures_iter(l)
            .map(|m| (format!("{}/issues/{}", url, m.get(1).unwrap().as_str()), local_msg.clone()))
            .collect::<Vec<(String, String)>>()
    }
}

pub struct RemoteRefExtractor {
    remote_ref_regex: Regex,
}

impl RemoteRefExtractor {
    pub fn new() -> Self {
        RemoteRefExtractor { remote_ref_regex: Regex::new(r" ([^/]+/[^#]+)#([0-9]*)").unwrap() }
    }

    pub fn extract(&self, l: &str) -> Vec<(String, String)> {
        let remote_msg = self.remote_ref_regex.replace(l, "").to_string().clone();
        self.remote_ref_regex
            .captures_iter(l)
            .map(|m| (format!("https://github.com/{}/issues/{}", m.get(1).unwrap().as_str(), m.get(2).unwrap().as_str()), remote_msg.clone()))
            .collect::<Vec<(String, String)>>()
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_remote_ref() {
        assert_eq!(
            RemoteRefExtractor::new().extract("ref org/repo#1375: some issue"),
            vec![("https://github.com/org/repo/issues/1375".to_string(), "ref: some issue".to_string())]
        );
    }
}