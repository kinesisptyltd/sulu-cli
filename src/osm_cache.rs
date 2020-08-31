use std::{
    fmt,
    collections::{
        HashMap,
        BTreeMap
    },
    io::{
        Read,
        Seek
    }
};
use osmpbfreader::{
    OsmPbfReader,
    OsmId,
    OsmObj,
    Node,
};
use serde::{Serialize, Deserialize};
use crate::{
    graph_config::GraphConfig,
    pbf_reader::{
        count_nodes,
        make_graph,
        Edge
    }
};

#[derive(Debug)]
pub enum Error {
    PbfError(osmpbfreader::error::Error),
    IoError(std::io::Error),
    NodeCountError,
    MakeGraphError
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IoError(e) => 
                write!(f, "IoError: {}", e),
            Error::PbfError(e) =>
                write!(f, "PbfError: {}", e),
            Error::NodeCountError => 
                write!(f, "Error counting nodes"),
            Error::MakeGraphError =>
                write!(f, "Error making graph")
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct OSMCache {
    pub graph_config: GraphConfig,
    pub osm_cache: BTreeMap<OsmId, OsmObj>,
    pub node_count: HashMap<OsmId, usize>
}

impl OSMCache {
    pub fn new(gc: GraphConfig) -> Self {
        OSMCache {
            graph_config: gc,
            osm_cache: BTreeMap::new(),
            node_count: HashMap::new()
        }
    }

    pub fn load_from_path<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(), Error> {
        let f = std::fs::File::open(path)
            .map_err(Error::IoError)?;
        let mut pbf = OsmPbfReader::new(f);
        self.load_pbf(&mut pbf)
    }

    pub fn load_pbf<R>(&mut self, pbf: &mut OsmPbfReader<R>) -> Result<(), Error> 
    where 
        R: Read + Seek
    {
        let now = std::time::Instant::now();
        let mut cache = pbf.get_objs_and_deps(
            |o| {
                match o {
                    OsmObj::Way(w) => self.graph_config.is_match(&w.tags),
                    _ => false
                }
            }
        ).map_err(Error::PbfError)?;
        println!("Extracted OSM objects in {}s", now.elapsed().as_secs());
        let now = std::time::Instant::now();
        let node_count = count_nodes(&cache)
            .map_err(|_| Error::NodeCountError)?;
        println!("Performed node count in {}s", now.elapsed().as_secs());
        self.osm_cache.append(&mut cache);
        for (k, v) in node_count.into_iter() {
            match self.node_count.get_mut(&k) {
                Some(ev) => { *ev = (*ev).max(v); },
                None => { self.node_count.insert(k, v); }
            }
        }
        Ok(())
    }

    pub fn make_graph(&self) -> Result<Vec<Edge>, Error> {
        make_graph(&self.osm_cache, &self.node_count)
            .map_err(|_| Error::MakeGraphError)
    }

    pub fn make_graph_bbox(&self, bbox: [f64; 4]) -> Result<Vec<Edge>, Error> {
        let osm_objs: BTreeMap<OsmId, OsmObj> = self.osm_cache
            .iter()
            .filter_map(
                |(i, obj)| {
                    match obj {
                        OsmObj::Way(w) => {
                            if w.nodes
                                .iter()
                                .filter_map(|nid| self.osm_cache.get(&OsmId::from(*nid)))
                                .filter_map(|o| o.node())
                                .any(|n| in_bbox(n, &bbox)) {
                                Some((*i, obj.clone()))
                            } else {
                                None
                            }
                        },
                        OsmObj::Node(n) => {
                            if in_bbox(&n, &bbox) {
                                Some((*i, obj.clone()))
                            } else {
                                None
                            }
                        },
                        _ => None
                    }
                }
            )
            .collect();
        make_graph(&osm_objs, &self.node_count)
            .map_err(|_| Error::MakeGraphError)
    }
}


fn in_bbox(node: &Node, bbox: &[f64; 4]) -> bool {
    (node.lon() >= bbox[0]) && (node.lat() >= bbox[1]) && (node.lon() <= bbox[2])
        && (node.lat() <= bbox[3])
}
