use std::{
    collections::{
        BTreeMap,
        HashMap
    },
    io::Read
};
use osmpbfreader::{
    OsmPbfReader,
    OsmId,
    OsmObj
};
use crate::{
    graph_config::GraphConfig,
    geom::BBox,
    error::Error,
    edge_list::{
        Edge,
        EdgeList
    }
};

pub trait GraphExtract {
    fn edge_list(&self, cache: &BTreeMap<OsmId, OsmObj>) -> Result<EdgeList, Error>;

    fn load_pbf<R>(
        &self, 
        pbf: &mut OsmPbfReader<R>, 
        bbox: &BBox,
        cache: &mut BTreeMap<OsmId, OsmObj>
    ) -> Result<(), Error>
    where 
        R: Read;
}

impl GraphExtract for GraphConfig {
    fn edge_list(&self, cache: &BTreeMap<OsmId, OsmObj>) -> Result<EdgeList, Error> {
        // count nodes
        let mut node_count = HashMap::new();
        for (_, obj) in cache {
            match obj {
                OsmObj::Way(way) => {
                    for nid in &way.nodes {
                        let nid: OsmId = (*nid).into();
                        match node_count.get_mut(&nid) {
                            Some(c) => *c += 1,
                            None => { node_count.insert(nid, 1); ()}
                        }
                    }
                },
                _ => ()
            }
        }
        // make edges
        let edges = cache.iter()
            .map(|(i, _)| Edge::from_osm_data(i, cache, &node_count))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|x| x.is_some())
            .collect::<Option<Vec<_>>>()
            .unwrap_or(Vec::new())
            .into_iter()
            .flatten()
            .collect::<Vec<Edge>>();
        // return
        Ok(EdgeList(edges))
    }

    fn load_pbf<R: Read>(
        &self, 
        pbf: &mut OsmPbfReader<R>, 
        bbox: &BBox,
        cache: &mut BTreeMap<OsmId, OsmObj>
    ) -> Result<(), Error>
    {
        let mut new_cache: BTreeMap<OsmId, OsmObj> = pbf.par_iter()
            .map(|ro| {
                ro.map(|o| {
                    match o {
                        OsmObj::Way(ref w) => {
                            if self.is_match(&w.tags) {
                                Some((o.id(), o))
                            } else {
                                None
                            }
                        },
                        OsmObj::Node(ref n) => {
                            if bbox.contains(&n.into()) {
                                Some((o.id(), o))
                            } else {
                                None
                            }
                        }
                        _ => None
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::PbfError)?
            .into_iter()
            .filter(|x| x.is_some())
            .collect::<Option<BTreeMap<_, _>>>()
            .unwrap_or(BTreeMap::new());
        cache.append(&mut new_cache);
        Ok(())
    }
}
