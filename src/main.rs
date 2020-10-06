use std::convert::TryInto;
use clap::{
    Arg,
    App,
};
use serde_json::from_reader;
use sulu::{
    graph_config::GraphConfig,
    osm_cache::OSMCache,
    edge_list::EdgeList,
    formats::Format
};

fn main() {
    let matches = App::new("Sulu")
        .version("0.1.0")
        .author("Tom Watson <tom.watson@kinesis.org>")
        .about("Converts osm.pbf files into routable networks")
        .arg(Arg::with_name("INPUT")
             .help("The osm.pbf file to process")
             .required(true)
             .index(1))
        .arg(Arg::with_name("OUTPUT")
             .help("The output file")
             .required(true)
             .index(2))
        .arg(Arg::with_name("GRAPH-CONFIG")
             .required(true)
             .help("File containing the definition of the graph")
             .index(3))
        .arg(Arg::with_name("format")
             .short("f")
             .help("How sulu will write the output file")
             .takes_value(true)
             .possible_values(&["geojson", "gdal"])
             .display_order(1)
             .default_value("geojson"))
        .arg(Arg::with_name("driver")
             .short("d")
             .takes_value(true)
             .default_value("gpkg")
             .help("Short name of gdal driver to use to write the output. See https://gdal.org/drivers/vector/index.html"))
        .get_matches();

    let graph_config_path = matches.value_of("GRAPH-CONFIG")
        .expect("No value for GRAPH-CONFIG");
    let graph_config_file = std::fs::File::open(graph_config_path).unwrap();
    let input_file_path = matches.value_of("INPUT")
        .expect("No value for INPUT");
    let output_path = matches.value_of("OUTPUT")
        .expect("No value for OUTPUT");

    let graph_config: GraphConfig = from_reader(graph_config_file).unwrap();

    let mut osm_cache = OSMCache::new(graph_config);
    osm_cache.load_from_path(input_file_path).unwrap();

    let edge_list: EdgeList<f64> = osm_cache.try_into().unwrap();

    match matches.value_of("format") {
        Some("geojson") => {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output_path)
                .unwrap();
            let format = Format::GeoJson(file);
            format.write(edge_list).unwrap();
        },
        Some("gdal") => {
            let driver_name = matches.value_of("driver")
                .unwrap_or("gpkg");
            let driver = gdal::Driver::get(driver_name).unwrap();
            let output_path = std::path::Path::new(output_path)
                .to_str()
                .expect("Not a valid path");
            let dataset = driver.create_vector_only(output_path).unwrap();
            let format = Format::Gdal(dataset);
            format.write(edge_list).unwrap();
        },
        _ => panic!("Unexepcted format")
    };
}
