use std::{
    fs::File,
    collections::{
        HashMap,
        BTreeMap
    }
};
use osmpbfreader::{
    OsmPbfReader,
    OsmId,
    OsmObj,
    Way,
};
use geo_types::{
    LineString,
    Coordinate
};
use crate::graph_config::{
    GraphConfig,
};

#[derive(Debug, Clone)]
pub enum Error {
    ExtractError,
    RewindError
}

pub struct Extracter {
    pub graph_config: GraphConfig,
    pub bbox: [f64; 4],
    osm_cache: BTreeMap<OsmId, OsmObj>
}

impl Extracter {
    pub fn new(graph_config: GraphConfig, bbox: [f64; 4]) -> Self {
        Extracter {
            graph_config: graph_config,
            bbox: bbox,
            osm_cache: BTreeMap::new()
        }
    }

    pub fn cache_nodes(&mut self, pbf: &mut OsmPbfReader<File>) -> Result<(), Error> {
        let mut node_map = pbf.get_objs_and_deps(
            |o| {
                match o {
                    OsmObj::Node(n) => {
                        (n.lon() >= self.bbox[0]) &&
                        (n.lat() >= self.bbox[1]) &&
                        (n.lon() <= self.bbox[2]) &&
                        (n.lat() <= self.bbox[3])
                    },
                    _ => false
                }
            })
            .map_err(|_| Error::ExtractError)?;
        self.osm_cache.append(&mut node_map);
        Ok(())
    }

    fn contains_only_cached_nodes(&self, way: &Way) -> bool {
        way.nodes.iter().all(|nid| self.osm_cache.contains_key(&OsmId::from(*nid)))
    }

    fn matches_graph_config(&self, way: &Way) -> bool {
        self.graph_config.is_match(&way.tags)
    }

    pub fn cache_ways(&mut self, pbf: &mut OsmPbfReader<File>) -> Result<(), Error> {
        let mut way_map: BTreeMap<OsmId, OsmObj> = pbf.par_iter()
            .filter_map(|o| {
                let o = o.unwrap(); // I think error if reading fails??
                match o {
                    OsmObj::Way(w) => {
                        if self.contains_only_cached_nodes(&w) && self.matches_graph_config(&w) {
                            Some((OsmId::from(w.id), OsmObj::from(w)))
                        } else {
                            None
                        }
                    },
                    _ => None
                }
            })
            .collect();
        self.osm_cache.append(&mut way_map);
        Ok(())
    }

    pub fn create_edges(&self) -> Result<Vec<Edge>, Error> {
        let node_count = count_nodes(&self.osm_cache)?;
        make_graph(&self.osm_cache, &node_count)
    }
}

pub fn extract_matching_ways_and_deps(
    pbf: &mut OsmPbfReader<File>,
    graph: &GraphConfig
) -> Result<BTreeMap<OsmId, OsmObj>, Error> {
    pbf.get_objs_and_deps(|o| {
        o.is_way() && graph.is_match(&o.tags())
    }).map_err(|_| Error::ExtractError)
}

pub fn extract_in_bbox(
    pbf: &mut OsmPbfReader<File>,
    graph: &GraphConfig,
    bbox: [f64; 4]
) -> BTreeMap<OsmId, OsmObj> {
    pbf.par_iter()
        .map(|o| {
            let o = o.unwrap();
            match o {
                OsmObj::Way(w) => {
                    match graph.is_match(&w.tags) {
                        true => Some((OsmId::from(w.id), OsmObj::from(w))),
                        false => None
                    }
                },
                OsmObj::Node(n) => {
                    match (n.lon() >= bbox[0]) && (n.lat() >= bbox[1]) && (n.lon() <= bbox[2]) && (n.lat() <= bbox[3]) {
                        true => Some((OsmId::from(n.id), OsmObj::from(n))),
                        false => None
                    }
                }
                _ => None
            }
        })
        .filter(|e| e.is_some())
        .map(|e| e.unwrap())
        .collect()
}

#[derive(Debug)]
pub struct Edge {
    pub way_osmid: i64,
    pub start_node_id: i64,
    pub end_node_id: i64,
    pub geometry: LineString<f64>
}

impl Edge {
    pub fn from_id(
        osmid: &OsmId,
        objs: &BTreeMap<OsmId, OsmObj>,
        node_count: &HashMap<OsmId, usize>
    ) -> Option<Vec<Edge>> {
        let obj = objs.get(osmid)?;
        if !obj.is_way() { // if the id is not a way, can't make an edge
            return None
        }
        let way_osmid: i64 = osmid.inner_id();
        let nodes: Vec<OsmId> = obj.way()?
            .nodes
            .iter()
            .map(|n| (*n).into())
            .collect();
        if nodes.iter().any(|n| !objs.contains_key(n)) {
            // if any nodes don't have info them, can't make the edge
            return None
        }
        let mut edges: Vec<Edge> = Vec::new();
        let mut points: Vec<Coordinate<f64>> = Vec::new();
        let mut start = nodes.first()?.clone();
        for nid in &nodes {
            let node = objs.get(&nid)?.node()?;
            points.push(Coordinate { x: node.lon(), y: node.lat() });
            let ncount = node_count.get(&nid)?;
            if (ncount > &1) && (*nid != start) {
                let end = nid.clone();
                edges.push(
                    Edge {
                        way_osmid: way_osmid,
                        start_node_id: start.inner_id(),
                        end_node_id: end.inner_id(),
                        geometry: points.into()
                    }
                );
                start = end;
                points = vec![Coordinate { x: node.lon(), y: node.lat() }];
            }
        }
        let end = nodes.last()?.clone();
        if start != end {
            edges.push(
                Edge {
                    way_osmid: way_osmid,
                    start_node_id: start.inner_id(),
                    end_node_id: end.inner_id(),
                    geometry: points.into()
                }
            );
        }
        Some(edges)
    }
}

pub fn make_graph(
    objs: &BTreeMap<OsmId, OsmObj>,
    node_count: &HashMap<OsmId, usize>
) -> Result<Vec<Edge>, Error>
{
    let mut edges = Vec::new();
    for osmid in objs.keys() {
        match Edge::from_id(osmid, &objs, &node_count) {
            Some(mut es) => edges.append(&mut es),
            None => ()
        }
    }
    Ok(edges)
}

pub fn count_nodes(
    objs: &BTreeMap<OsmId, OsmObj>
) -> Result<HashMap<OsmId, usize>, Error> {
    let mut map = HashMap::new();
    for (_, obj) in objs {
        if obj.is_way() {
            let way = obj.way().unwrap();
            for nid in &way.nodes {
                let nid: OsmId = (*nid).into();
                match map.get_mut(&nid) {
                    Some(c) => *c += 1,
                    None => {map.insert(nid, 1); ()}
                }
            }
        }
    }
    Ok(map)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use serde_json;
// 
    //#[test]
    //fn extract() {
        //let file = File::open("/home/tom/src/sulu-rs/sydneyish.osm.pbf").unwrap();
        //let mut pbf = OsmPbfReader::new(file);
        //let s = r#"{
            //"name": "roads",
            //"options": [{
                //"name": "big-road",
                //"requires": [
                    //{"key": "highway", "kind": {"in-list": ["primary", "secondary"]}}
                //],
                //"excludes": [
                    //{"key": "access", "kind": {"exact": "no"}}
                //]
            //}]
        //}"#;
        //let gco: GraphConfig = serde_json::from_str(s).unwrap();
////        let objs = extract_matching_ways_and_deps(&mut pbf, &gco).unwrap();
        //let objs = extract_in_bbox(&mut pbf, &gco, [151.10584, -33.917765, 151.164958, -33.885377]);
        //println!(
            //"Extracted {} ways and {} nodes",
            //objs.iter().filter(|(_, o)| o.is_way()).count(),
            //objs.iter().filter(|(_, o)| o.is_node()).count()
        //);
        //let node_count = count_nodes(&objs).unwrap();
        //println!("node_count has {} entries", node_count.len());
        //let edges = make_graph(&objs, &node_count).unwrap();
        //println!("Number of edges: {}", edges.len());
    //}
// }
