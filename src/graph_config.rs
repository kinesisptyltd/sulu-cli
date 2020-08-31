use serde::{Serialize, Deserialize};
use serde_json;
use osmpbfreader::objects::Tags;
use crate::matcher::{Matcher};

#[serde(rename_all="kebab-case")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphConfigOption {
    pub name: String,
    pub requires: Vec<Matcher>,
    pub excludes: Vec<Matcher>
}

impl GraphConfigOption {
    pub fn check_match(&self, tags: &Tags) -> bool {
        !self.excludes
            .iter()
            .any(|m| tags
                 .iter()
                 .any(|(key, value)| m.match_tag(key, value))
            ) &&
        self.requires
            .iter()
            .all(|m| tags
                 .iter()
                 .any(|(key, value)| m.match_tag(key, value))
            )
    }
}

#[serde(rename_all="kebab-case")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphConfig {
    pub name: String,
    pub options: Vec<GraphConfigOption>
}

impl GraphConfig {
    pub fn matching_option(&self, tags: &Tags) -> Option<GraphConfigOption>
    {
        for opt in &self.options {
            if opt.check_match(tags) {
                return Some(opt.clone())
            }
        }
        None
    }

    pub fn is_match(&self, tags: &Tags) -> bool {
        for opt in &self.options {
            if opt.check_match(tags) {
                return true
            }
        }
        false
    }

    /// Make a graph config from a file of json
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let gc: Self = serde_json::from_str(&data)?;
        Ok(gc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_de() {
        let s = r#"{
            "name": "big-road",
            "requires": [
                {"key": "highway", "kind": {"in-list": ["primary", "secondary"]}}
            ],
            "excludes": [
                {"key": "access", "kind": {"exact": "no"}}
            ]
        }"#;
        let gco: GraphConfigOption = serde_json::from_str(s).unwrap();
        assert_eq!(gco.name, "big-road");

        let tags = vec![("highway".to_string(), "primary".to_string())];
        let tags2: Vec<(&String, &String)> = tags.iter().map(|(k,v)| (k, v)).collect();

        assert!(gco.check_match(&tags2));

        let tags = vec![
            ("highway".to_string(), "primary".to_string()),
            ("access".to_string(), "no".to_string())
        ];
        let tags2: Vec<(&String, &String)> = tags.iter().map(|(k,v)| (k, v)).collect();
        assert!(!gco.check_match(&tags2));
    }
}
