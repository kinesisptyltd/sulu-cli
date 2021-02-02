#![allow(dead_code)]
use std::{
    convert::From,
    collections::HashMap
};
use num_traits::float::Float;
use geo::{
    Coordinate,
    CoordinateType,
};


/// A visibility graph container type.
pub struct VisibilityGraph<N> {
    nodes: Vec<N>,
    edge_list: Vec<(N, N)>,
    adjacency_map: HashMap<N, Vec<N>>
}

impl<N> VisibilityGraph<N> {
    pub fn new() -> Self {
        VisibilityGraph {
            nodes: vec![],
            edge_list: vec![],
            adjacency_map: HashMap::new()
        }
    }
}

/// A node container for our visibility graph
pub struct Node<I, T: CoordinateType> {
    id: I,
    coordinate: Coordinate<T>
}

/// Like a LineString but with Nodes
pub struct NodeString<I, T: CoordinateType>(pub Vec<Node<I, T>>);


/// Like a Polygon but with NodeStrings
pub struct NodePolygon<I, T: CoordinateType> {
    exterior: NodeString<I, T>,
    interiors: Vec<NodeString<I, T>>
}


/// Trait for Node id field
pub trait IdType {}


impl<I, T> From<NodePolygon<I, T>> for VisibilityGraph<Node<I, T>> 
where
    I: IdType,
    T: CoordinateType + Float + std::fmt::Debug
{
    fn from(_node_poly: NodePolygon<I, T>) -> VisibilityGraph<Node<I, T>> {
        VisibilityGraph::new()
    }
}
