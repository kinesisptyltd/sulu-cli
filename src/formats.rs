use std::io::Write;
use geojson;
use serde_json::{
    json,
    Map,
    Value
};
use geo_types::LineString;
use gdal::{
    self,
    vector::FieldValue,
};
use crate::{
    error::Error,
    edge_list::{
        Edge,
        EdgeList
    }
};


pub enum Format {
    GeoJson(std::fs::File),
    Gdal(gdal::vector::Dataset)
}

impl Format {
    pub fn write(self, el: EdgeList<f64>) -> Result<(), Error> {
        match self {
            Format::GeoJson(mut file) => {
                let fc: geojson::FeatureCollection = el.into();
                let g: geojson::GeoJson = fc.into();
                file.write_all(&g.to_string().into_bytes())
                    .map_err(Error::IoError)?;
            },
            Format::Gdal(mut ds) => {
                let srs = gdal::spatial_ref::SpatialRef::from_epsg(4326)
                    .map_err(Error::GdalError)?;
                let layer = ds.create_layer_ext("graph",
                                                Some(&srs),
                                                gdal::vector::OGRwkbGeometryType::wkbLineString)
                    .map_err(Error::GdalError)?;
                layer.create_defn_fields(&[("way_osmid", gdal::vector::OGRFieldType::OFTString),
                                           ("start_node_id", gdal::vector::OGRFieldType::OFTString),
                                           ("end_node_id", gdal::vector::OGRFieldType::OFTString),
                                           ("graph_config_option", gdal::vector::OGRFieldType::OFTString),
                                           ("length_m", gdal::vector::OGRFieldType::OFTReal)])
                    .map_err(Error::GdalError)?;
                for edge in el.edges.iter() {
                    let geom = linestring_to_gdal(&edge.geometry)?;
                    let field_names = ["way_osmid", 
                                       "start_node_id", 
                                       "end_node_id", 
                                       "graph_config_option", 
                                       "length_m"];
                    // gdal 0.6.0 doesn't have an Integer64 FieldValue yet, but there's an open
                    // PR to implement it: https://github.com/georust/gdal/pull/80
                    // For now we'll just parse the OSM IDs to strings
                    let field_values = [FieldValue::StringValue(edge.way_osmid.0.to_string()),
                                        FieldValue::StringValue(edge.start_node_id.0.to_string()),
                                        FieldValue::StringValue(edge.end_node_id.0.to_string()),
                                        FieldValue::StringValue(edge.graph_config_option.name.clone()),
                                        FieldValue::RealValue(edge.length_m)];
                    layer.create_feature_fields(
                        geom,
                        &field_names,
                        &field_values)
                    .map_err(Error::GdalError)?;
                }
            }
        }
        Ok(())
    }
}


fn linestring_to_gdal(linestring: &LineString<f64>) -> Result<gdal::vector::Geometry, Error> {
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

impl From<EdgeList<f64>> for geojson::FeatureCollection {
    fn from(el: EdgeList<f64>) -> geojson::FeatureCollection {
        geojson::FeatureCollection {
            bbox: None,
            foreign_members: None,
            features: el.edges.into_iter().map(|e| e.into()).collect()
        }
    }
}

// impl From<Edge<f64>> for gdal::vector::Feature {
//     fn from(edge: Edge<f64>) -> gdal::vector::Feature {
//     }
// }
