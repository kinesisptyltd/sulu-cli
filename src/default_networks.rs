use crate::{
    graph_config::{
        GraphConfig,
        GraphConfigOption
    },
    matcher::{
        Matcher,
        MatchKind
    },
};
use std::str::FromStr;
use crate::error::Error;


#[derive(Debug, PartialEq)]
pub enum Network {
    Walk,
    Drive,
    Cycle
}

impl FromStr for Network {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "walk" => Ok(Network::Walk),
            "drive" => Ok(Network::Drive),
            "cycle" => Ok(Network::Cycle),
            _ => Err(Error::ConversionError(
                    format!("Unknown network: {}", s)))
        }
    }
}


#[cfg(feature="sulu-http")]
use rocket::{
    request::FromFormValue,
    http::RawStr,
};

#[cfg(feature="sulu-http")]
impl<'v> FromFormValue<'v> for Network {
    type Error = Error;

    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        Network::from_str(form_value.as_str())
    }

    fn default() -> Option<Self> {
        Some(Network::Walk)
    }
}

impl Network {
    pub fn definition(&self) -> GraphConfig {
        match self {
            Network::Walk => walk_network(),
            Network::Drive => drive_network(), // placeholder
            Network::Cycle => cycle_network() // placeholder
        }
    }
}

fn walk_network() -> GraphConfig {
    GraphConfig {
        name: "walk".to_string(),
        options: vec![
            GraphConfigOption {
                name: "foot".to_string(),
                requires: vec![
                    Matcher {
                        key: "foot".to_string(),
                        kind: MatchKind::InList(vec![
                                                "yes".to_string(),
                                                "designated".to_string()])
                    }
                ],
                excludes: vec![
                    Matcher {
                        key: "route".to_string(),
                        kind: MatchKind::Exact("ferry".to_string())
                    }
                ]
            },
            GraphConfigOption {
                name: "walkable_roads".to_string(),
                requires: vec![
                    Matcher {
                        key: "highway".to_string(),
                        kind: MatchKind::InList(vec![
                                                "trunk".to_string(),
                                                "primary".to_string(),
                                                "secondary".to_string(),
                                                "tertiary".to_string(),
                                                "unclassified".to_string(),
                                                "residential".to_string(),
                                                "trunk_link".to_string(),
                                                "primary_link".to_string(),
                                                "secondary_link".to_string(),
                                                "tertiary_link".to_string(),
                                                "service".to_string()])
                    }
                ],
                excludes: vec![
                    Matcher { key: "foot".to_string(), kind: MatchKind::Exact("no".to_string()) },
                    Matcher { 
                        key: "sidewalk".to_string(), 
                        kind: MatchKind::InList(vec![
                                                "no".to_string(),
                                                "none".to_string(),
                                                "separate".to_string()])
                    },
                    Matcher {
                        key: "access".to_string(),
                        kind: MatchKind::InList(vec![
                                                "no".to_string(),
                                                "private".to_string()])
                    }
                ]
            },
            GraphConfigOption {
                name: "other_walkable_highway".to_string(),
                requires: vec![
                    Matcher {
                        key: "highway".to_string(),
                        kind: MatchKind::InList(vec![
                                                "pedestrian".to_string(),
                                                "footway".to_string(),
                                                "living_street".to_string(),
                                                "path".to_string(),
                                                "track".to_string(),
                                                "steps".to_string(),
                                                "cycleway".to_string()])
                    }
                ],
                excludes: vec![
                    Matcher {
                        key: "access".to_string(),
                        kind: MatchKind::InList(vec![
                                                "no".to_string(),
                                                "private".to_string()])
                    },
                    Matcher { key: "area".to_string(), kind: MatchKind::Exact("yes".to_string()) },
                    Matcher { key: "foot".to_string(), kind: MatchKind::Exact("no".to_string()) }
                ]
            },
            GraphConfigOption {
                name: "crossing".to_string(),
                requires: vec![Matcher { key: "footway".to_string(), kind: MatchKind::Exact("crossing".to_string())}],
                excludes: vec![]
            },
        ]
    }
}


fn drive_network() -> GraphConfig {
    GraphConfig {
        name: "drive".to_string(),
        options: vec![
            GraphConfigOption {
                name: "highway".to_string(),
                requires: vec![
                    Matcher {
                        key: "highway".to_string(),
                        kind: MatchKind::InList(vec![
                                                "motorway".to_string(),
                                                "motorway_link".to_string(),
                                                "trunk".to_string(),
                                                "trunk_link".to_string(),
                                                "primary".to_string(),
                                                "primary_link".to_string(),
                                                "secondary".to_string(),
                                                "secondary_link".to_string(),
                                                "tertiary".to_string(),
                                                "tertiary_link".to_string(),
                                                "unclassified".to_string(),
                                                "residential".to_string(),
                                                "living_street".to_string(),
                                                "road".to_string()])
                    }
                ],
                excludes: vec![]
            },
            GraphConfigOption { 
                name: "service".to_string(),
                requires: vec![
                    Matcher {
                        key: "highway".to_string(),
                        kind: MatchKind::Exact("service".to_string())
                    }
                ],
                excludes: vec![
                    Matcher {
                        key: "access".to_string(),
                        kind: MatchKind::InList(vec!["no".to_string(), "private".to_string()])
                    }
                ]
            },
            GraphConfigOption {
                name: "track".to_string(),
                requires: vec![
                    Matcher {
                        key: "highway".to_string(), 
                        kind: MatchKind::Exact("track".to_string())
                    }
                ],
                excludes: vec![
                    Matcher {
                        key: "access".to_string(),
                        kind: MatchKind::InList(vec![
                                                "no".to_string(),
                                                "private".to_string()])
                    },
                    Matcher {
                        key: "vehicle".to_string(),
                        kind: MatchKind::Exact("no".to_string())
                    },
                    Matcher {
                        key: "motor_vehicle".to_string(),
                        kind: MatchKind::Exact("no".to_string())
                    }
                ]
            },
        ]
    }
}


fn cycle_network() -> GraphConfig {
    GraphConfig {
        name: "drive".to_string(),
        options: vec![]
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_guard() {
        let x: Network = FromFormValue::from_form_value("walk".into())
            .expect("Expected a network enum");
        assert_eq!(x, Network::Walk);
        let x: Network = FromFormValue::from_form_value("drive".into())
            .expect("Expected a network enum");
        assert_eq!(x, Network::Drive);
        let x: Network = FromFormValue::from_form_value("cycle".into())
            .expect("Expected a network enum");
        assert_eq!(x, Network::Cycle);
    }
}
