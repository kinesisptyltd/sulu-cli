/// Types for representing a graph as an edge list

use std::convert::TryFrom;
use geo::{
    LineString,
    Coordinate,
    CoordNum,
    algorithm::geodesic_length::GeodesicLength,
};
use osmpbfreader::{OsmId, WayId, NodeId};
use crate::{
    graph_config::GraphConfigOption,
    osm_cache::OSMCache,
    error::Error,
};
#[cfg(feature="formats-geojson")]
use serde_json::{json, Value, Map};
#[cfg(feature="formats-geojson")]
use geojson;
#[cfg(feature="formats-gdal")]
use gdal;

#[derive(Debug)]
pub struct Edge<T: CoordNum> {
    pub way_osmid: WayId,
    pub start_node_id: NodeId,
    pub end_node_id: NodeId,
    pub graph_config_option: GraphConfigOption,
    pub length_m: f64,
    pub geometry: LineString<T>
}

#[derive(Debug)]
pub struct EdgeList<T: CoordNum> {
    pub edges: Vec<Edge<T>>
}

impl TryFrom<OSMCache> for EdgeList<f64> 
{
    type Error = Error;

    fn try_from(osm_cache: OSMCache) -> Result<Self, Self::Error> {
        let edgelist = osm_cache.osm_cache.keys()
            .filter(|o| o.is_way())
            .map(|o| edges_from_osm_id(o, &osm_cache))
            .collect::<Result<Vec<Option<_>>, _>>()?
            .into_iter()
            .filter(|x| x.is_some())
            .collect::<Option<Vec<_>>>()
            .map(|v| EdgeList { edges : v.into_iter()
                .flatten()
                .collect() })
            .unwrap_or(EdgeList { edges : vec![] });
        Ok(edgelist)
    }
}

fn edges_from_osm_id(
    osmid: &OsmId,
    osm_cache: &OSMCache
) -> Result<Option<Vec<Edge<f64>>>, Error> {
    let objs = &osm_cache.osm_cache;
    let node_count = &osm_cache.node_count;
    let way = match objs.get(osmid)
        .ok_or(Error::MissingInfo(format!("OSM id not in cache: {:?}", osmid)))?
        .way() {
            Some(w) => w,
            None => return Ok(None),
        };
    let gco = osm_cache.graph_config
        .matching_option(&way.tags)
        .ok_or(Error::MissingInfo(format!("Way doesn't match the graph config: {:?}", osmid)))?;
    let max_edges = way.nodes.len();
    let mut edges: Vec<Edge<f64>> = Vec::with_capacity(max_edges);
    let mut points: Vec<Coordinate<f64>> = Vec::with_capacity(max_edges);
    let mut start: Option<NodeId> = None;
    let mut end: Option<NodeId> = None;
    for nid in way.nodes.iter() {
        let node_osmid = nid.clone().into();
        let count = node_count.get(&node_osmid).unwrap_or(&1);
        match objs.get(&node_osmid) {
            // start or continue edge
            Some(obj) => {
                end = Some(*nid);
                let node = obj.node().ok_or(Error::NotANode(obj.clone()))?;
                match start {
                    // continue edge if count is 1, otherwise end edge and start a new one
                    Some(sid) => {
                        let coords = Coordinate { x: node.lon().into(), y: node.lat().into() };
                        points.push(coords);
                        if (count > &1) && start != end {
                            points.shrink_to_fit();
                            let geom: LineString<f64> = points.clone().into();
                            edges.push( Edge {
                                way_osmid: osmid.way().ok_or(Error::NotAWayId(*osmid))?,
                                start_node_id: sid,
                                end_node_id: end.unwrap_or(nid.clone()),
                                graph_config_option: gco.clone(),
                                length_m: geom.geodesic_length(),
                                geometry: geom
                            });
                            points = Vec::with_capacity(max_edges);
                            points.push(coords);
                            start = Some(*nid);
                            end = None;
                        }
                    },
                    // start new edge
                    None =>  {
                        points = Vec::with_capacity(max_edges);
                        let coords = Coordinate { x: node.lon().into(), y: node.lat().into() };
                        points.push(coords);
                        start = Some(*nid);
                        end = None;
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
                            let geom: LineString<f64> = points.clone().into();
                            edges.push( Edge {
                                way_osmid: osmid.way().ok_or(Error::NotAWayId(*osmid))?,
                                start_node_id: sid,
                                end_node_id: end.unwrap_or(nid.clone()),
                                graph_config_option: gco.clone(),
                                length_m: geom.geodesic_length(),
                                geometry: geom
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
    // Finalise the last edge
    match end {
        // There's a dangling edge, end it
        Some(nid) => {
            let geom: LineString<f64> = points.clone().into();
            edges.push( Edge {
                way_osmid: osmid.way().ok_or(Error::NotAWayId(*osmid))?,
                start_node_id: start.ok_or(Error::MakeGraphError)?,
                end_node_id: nid,
                graph_config_option: gco.clone(),
                length_m: geom.geodesic_length(),
                geometry: geom
            });
        },
        // There's no dangling edge, continue
        None => () 
    }

    edges.shrink_to_fit();
    Ok(Some(edges))
}


#[cfg(feature="formats-geojson")]
impl From<Edge<f64>> for geojson::Feature {
    fn from(edge: Edge<f64>) -> geojson::Feature {
        let mut props: Map<String, Value> = Map::new();
        props.insert("way_osmid".to_string(), json!(edge.way_osmid));
        props.insert("start_node_id".to_string(), json!(edge.start_node_id));
        props.insert("end_node_id".to_string(), json!(edge.end_node_id));
        props.insert("graph_config_option".to_string(), json!(edge.graph_config_option.name));
        props.insert("length_m".to_string(), json!(edge.length_m));
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

#[cfg(feature="formats-geojson")]
impl From<EdgeList<f64>> for geojson::FeatureCollection {
    fn from(el: EdgeList<f64>) -> geojson::FeatureCollection {
        geojson::FeatureCollection {
            bbox: None,
            foreign_members: None,
            features: el.edges.into_iter().map(|e| e.into()).collect()
        }
    }
}

#[cfg(feature="formats-gdal")]
pub fn linestring_to_gdal(linestring: &LineString<f64>) -> Result<gdal::vector::Geometry, Error> {
    // gdal version 0.6.0 relies on geo-types 0.4.0 (not 0.6.0), but I don't want to change
    // versions of geo-types, so we can just hack this in here, and use the ToGdal
    // trait when gdal catches up
    let mut geom = gdal::vector::Geometry::empty(gdal::vector::OGRwkbGeometryType::wkbLineString)
        .map_err(Error::GdalError)?;
    for (i, pt) in linestring.points_iter().enumerate() {
        geom.set_point_2d(i, (pt.x(), pt.y()));
    }
    Ok(geom)
}
