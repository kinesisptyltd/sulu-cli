use std::{
    io::Write,
    default::Default,
};
use geojson;
#[cfg(feature="formats-gdal")]
use gdal::{
    self,
    vector::FieldValue,
};
use sulu_lib::{
    error::Error,
    edge_list::{
        EdgeList,
    }
};
#[cfg(feature="formats-gdal")]
use sulu_lib::edge_list::linestring_to_gdal;


pub enum Format {
    GeoJson(std::fs::File),
    #[cfg(feature="formats-gdal")]
    Gdal(gdal::Dataset)
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
            #[cfg(feature="formats-gdal")]
            Format::Gdal(mut ds) => {
                let srs = gdal::spatial_ref::SpatialRef::from_epsg(4326)
                    .map_err(Error::GdalError)?;
                let mut layer = ds.create_layer(
                    gdal::LayerOptions { 
                        name: "graph",
                        srs: Some(&srs),
                        ty: gdal::vector::OGRwkbGeometryType::wkbLineString,
                        ..Default::default()
                    })
                    .map_err(Error::GdalError)?;
                layer.create_defn_fields(&[("way_osmid", gdal::vector::OGRFieldType::OFTInteger64),
                                           ("start_node_id", gdal::vector::OGRFieldType::OFTInteger64),
                                           ("end_node_id", gdal::vector::OGRFieldType::OFTInteger64),
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
                    let field_values = [FieldValue::Integer64Value(edge.way_osmid.0),
                                        FieldValue::Integer64Value(edge.start_node_id.0),
                                        FieldValue::Integer64Value(edge.end_node_id.0),
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
