pub mod matcher;
pub mod graph_config;
pub mod error;
pub mod edge_list;
pub mod osm_cache;
pub mod formats;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
