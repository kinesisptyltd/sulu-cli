pub mod matcher;
pub mod graph_config;
pub mod pbf_reader;
pub mod osm_cache;
pub mod error;
pub mod geom;
pub mod graph_extract;
pub mod edge_list;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
