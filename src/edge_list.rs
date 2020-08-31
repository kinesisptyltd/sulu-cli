/// Types for representing a graph as an edge list

use std::collections::{BTreeMap, HashMap};
use geo::{
    LineString,
    Coordinate,
    algorithm::geodesic_length::GeodesicLength,
};
use geojson;
use serde_json::{Map, json, Value};
use osmpbfreader::{OsmId, OsmObj};
use crate::error::Error;

#[derive(Debug)]
pub struct Edge {
    pub way_osmid: i64,
    pub start_node_id: i64,
    pub end_node_id: i64,
    pub geometry: LineString<f64>
}

impl Edge {
    pub fn from_osm_data(
        osmid: &OsmId,
        objs: &BTreeMap<OsmId, OsmObj>,
        node_count: &HashMap<OsmId, usize>
    ) -> Result<Option<Vec<Edge>>, Error> {
        let way = match objs.get(osmid)
            .ok_or(Error::MissingInfo(format!("OSM id not in cache: {:?}", osmid)))?
            .way() {
                Some(w) => w,
                None => return Ok(None),
            };
        let max_edges = way.nodes.len();
        let mut edges: Vec<Edge> = Vec::with_capacity(max_edges);
        let mut points: Vec<Coordinate<f64>> = Vec::with_capacity(max_edges);
        let mut start: Option<i64> = None;
        let mut end: Option<i64> = None;
        for nid in way.nodes.iter() {
            let node_osmid = nid.clone().into();
            let count = node_count.get(&node_osmid).unwrap_or(&1);
            match objs.get(&node_osmid) {
                // start or continue edge
                Some(obj) => {
                    end = Some(nid.0);
                    let node = obj.node().ok_or(Error::NotANode(obj.clone()))?;
                    match start {
                        // continue edge if count is 1, otherwise end edge and start a new one
                        Some(sid) => {
                            let coords = Coordinate { x: node.lon(), y: node.lat() };
                            points.push(coords);
                            if (count > &1) && (points.len() > 1) {
                                points.shrink_to_fit();
                                edges.push( Edge {
                                    way_osmid: osmid.inner_id(),
                                    start_node_id: sid,
                                    end_node_id: end.unwrap_or(nid.0.clone()),
                                    geometry: points.clone().into()
                                });
                                points = Vec::with_capacity(max_edges);
                                points.push(coords);
                                start = Some(nid.0);
                            }
                        },
                        // start new edge
                        None =>  {
                            points = Vec::with_capacity(max_edges);
                            let coords = Coordinate { x: node.lon(), y: node.lat() };
                            points.push(coords);
                            start = Some(nid.0);
                        }
                    }
                },
                // skip or end edge
                None => {
                    match start {
                        // end edge
                        Some(sid) => {
                            if points.len() > 1 {
                                points.shrink_to_fit();
                                edges.push( Edge {
                                    way_osmid: osmid.inner_id(),
                                    start_node_id: sid,
                                    end_node_id: end.unwrap_or(nid.0.clone()),
                                    geometry: points.clone().into()
                                });
                            }
                            points = Vec::with_capacity(max_edges);
                            start = None;
                            end = None;
                        },
                        // skip
                        None => ()
                    }
                }
            }
        }
        edges.shrink_to_fit();
        Ok(Some(edges))
    }
}

impl From<Edge> for geojson::Feature {
    fn from(edge: Edge) -> geojson::Feature {
        let mut props: Map<String, Value> = Map::new();
        props.insert("way_osmid".to_string(), json!(edge.way_osmid));
        props.insert("start_node_id".to_string(), json!(edge.start_node_id));
        props.insert("end_node_id".to_string(), json!(edge.end_node_id));
        props.insert("length_m".to_string(), json!(edge.geometry.geodesic_length()));
        let geom = geojson::Geometry {
            bbox: None,
            foreign_members: None,
            value: (&edge.geometry).into()
        };
        geojson::Feature {
            bbox: None,
            geometry: Some(geom),
            id: None,
            properties: Some(props),
            foreign_members: None
        }
    }
}

impl From<EdgeList> for geojson::FeatureCollection {
    fn from(el: EdgeList) -> geojson::FeatureCollection {
        geojson::FeatureCollection {
            bbox: None,
            foreign_members: None,
            features: el.0.into_iter().map(|e| e.into()).collect()
        }
    }
}

#[derive(Debug)]
pub struct EdgeList(pub Vec<Edge>);
