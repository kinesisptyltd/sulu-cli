pub mod formats;

use std::convert::TryInto;
use clap::{
    Arg,
    Command
};
use serde_json::from_reader;
use crate::formats::Format;
use sulu_lib::{
    graph_config::GraphConfig,
    osm_cache::OSMCache,
    edge_list::EdgeList,
};


fn cli() -> Command {
    let app = Command::new("Sulu")
        .version("0.2.0")
        .author("Tom Watson <tom.watson@kinesis.org>")
        .about("Converts osm.pbf files into routable networks")
        .arg(Arg::new("INPUT")
             .help("The osm.pbf file to process")
             .required(true)
             .index(1))
        .arg(Arg::new("OUTPUT")
             .help("The output file")
             .required(true)
             .index(2))
        .arg(Arg::new("GRAPH-CONFIG")
             .required(true)
             .help("File containing the definition of the graph")
             .index(3));

    #[cfg(feature="formats-gdal")]
    let app = app.clone().arg(Arg::new("gdal-driver")
                .long("gdal-driver")
                .short('d')
                .help("Use gdal to output file with a specific driver")
                .action(clap::ArgAction::Set));

    app
}

fn main() {
    let matches = cli().get_matches();

    let graph_config_path = matches.get_one::<String>("GRAPH-CONFIG")
        .expect("No value for GRAPH-CONFIG");
    let graph_config_file = std::fs::File::open(graph_config_path).unwrap();
    let input_file_path = matches.get_one::<String>("INPUT")
        .expect("No value for INPUT");

    let graph_config: GraphConfig = from_reader(graph_config_file).unwrap();

    let mut osm_cache = OSMCache::new(graph_config);
    osm_cache.load_from_path(input_file_path).unwrap();

    let edge_list: EdgeList<f64> = osm_cache.try_into().unwrap();

    match matches.get_one::<String>("gdal-driver") {
        #[cfg(feature="formats-gdal")]
        Some(driver_name) => {
            let output_path = matches.get_one::<String>("OUTPUT")
                .expect("No value for OUTPUT");
            let driver = gdal::DriverManager::get_driver_by_name(driver_name)
                .expect("Not a valid driver name, see https://gdal.org/drivers/vector/index.html");
            let output_path = std::path::Path::new(output_path)
                .to_str()
                .expect("Not a valid path");
            let dataset = driver.create_vector_only(output_path).unwrap();
            let format = Format::Gdal(dataset);
            format.write(edge_list).unwrap();
        },
        _ => {
            let output_path = matches.get_one::<String>("OUTPUT")
                .expect("No value for OUTPUT");
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output_path)
                .unwrap();
            let format = Format::GeoJson(file);
            format.write(edge_list).unwrap();
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn cli_definition_is_valid() {
        cli().debug_assert();
    }

    #[test]
    fn cli_parses_positionals_in_order() {
        let m = cli().try_get_matches_from(["sulu", "in.osm.pbf", "out.geojson", "graph.json"]).unwrap();
        assert_eq!(m.get_one::<String>("INPUT").unwrap(), "in.osm.pbf");
        assert_eq!(m.get_one::<String>("OUTPUT").unwrap(), "out.geojson");
        assert_eq!(m.get_one::<String>("GRAPH-CONFIG").unwrap(), "graph.json");
    }

    #[test]
    fn cli_requires_graph_config() {
        let err = cli().try_get_matches_from(["sulu", "in.osm.pbf", "out.geojson"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[cfg(feature = "formats-gdal")]
    #[test]
    fn cli_parses_gdal_driver_in_long_and_short_form() {
        for flag in ["--gdal-driver", "-d"] {
            let m = cli()
                .try_get_matches_from(["sulu", flag, "GPKG", "in.osm.pbf", "out.gpkg", "graph.json"])
                .unwrap();
            assert_eq!(m.get_one::<String>("gdal-driver").unwrap(), "GPKG", "{} not recognised", flag);
        }
    }

    #[cfg(not(feature = "formats-gdal"))]
    #[test]
    fn cli_rejects_gdal_driver_without_gdal_support() {
        let err = cli()
            .try_get_matches_from(["sulu", "-d", "GPKG", "in.osm.pbf", "out.gpkg", "graph.json"])
            .unwrap_err();
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }
}
