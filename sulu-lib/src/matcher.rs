use serde::{Serialize, Deserialize};
use smartstring::alias::String;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all="kebab-case")]
pub enum MatchKind {
    InList(Vec<String>),
    Exact(String),
    All
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all="kebab-case")]
pub struct Matcher {
    pub key: String,
    pub kind: MatchKind
}

impl Matcher {
    pub fn match_tag(&self, key: &String, value: &String) -> bool {
        if self.key == *key {
            return match &self.kind {
                MatchKind::All => true,
                MatchKind::Exact(v) => v == value,
                MatchKind::InList(vs) => vs.iter().any(|v| v == value)
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_de() {
        let s = "{\"key\": \"highway\", \"kind\": \"all\"}";
        let m: Matcher = serde_json::from_str(s).unwrap();
        assert_eq!(m.key.as_str(), "highway");
        assert!(match m.kind {
            MatchKind::All => true,
            _ => false
        });

        let s = "{\"key\": \"highway\", \"kind\": {\"exact\": \"primary\"}}";
        let m: Matcher = serde_json::from_str(s).unwrap();
        assert_eq!(m.key.as_str(), "highway");
        assert!(match m.kind {
            MatchKind::Exact(v) => v.as_str() == "primary",
            _ => false
        });

        let s = "{\"key\": \"highway\", \"kind\": {\"in-list\": [\"primary\", \"secondary\"]}}";
        let m: Matcher = serde_json::from_str(s).unwrap();
        assert_eq!(m.key.as_str(), "highway");
        assert!(match m.kind {
            MatchKind::InList(v) => v.contains(&"primary".into()),
            _ => false
        });
    }
}
