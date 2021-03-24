use std::{
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
};
use serde::{Serialize, Deserialize};
use crate::{
    graph_config::GraphConfig,
    error::Error,
};

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
        let mut cache = pbf.get_objs_and_deps(
            |o| {
                match o {
                    OsmObj::Way(w) => self.graph_config.is_match(&w.tags),
                    _ => false
                }
            }
        ).map_err(Error::PbfError)?;
        let node_count = count_nodes(&cache)
            .map_err(|_| Error::NodeCountError)?;
        self.osm_cache.append(&mut cache);
        for (k, v) in node_count.into_iter() {
            match self.node_count.get_mut(&k) {
                Some(ev) => { *ev = (*ev).max(v); },
                None => { self.node_count.insert(k, v); }
            }
        }
        Ok(())
    }
}
