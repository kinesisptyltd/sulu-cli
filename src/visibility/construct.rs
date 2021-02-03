#![allow(dead_code)]
use std::{
    convert::From,
    collections::HashMap
};
use num_traits::float::Float;
use geo::{
    Coordinate,
    CoordinateType,
    Line,
    algorithm::intersects::Intersects
};
use super::rotational_sweep::{
    CoordTrait,
    LineTrait,
    rotational_sweep,
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
#[derive(Debug, Clone, PartialEq)]
pub struct Node<I, T: CoordinateType> {
    id: I,
    coordinate: Coordinate<T>
}

impl<I, T: CoordinateType> CoordTrait<T> for Node<I, T> {
    fn coord(&self) -> Coordinate<T> {
        self.coordinate
    }
}

pub struct NodeLine<N> {
    start: N,
    end: N
}

impl<N> NodeLine<N> {
    pub fn new(start: N, end: N) -> Self {
        NodeLine { start: start,
                   end: end }
    }
}

impl<I, T: CoordinateType> LineTrait<T> for NodeLine<Node<I, T>> {
    fn line(&self) -> Line<T> {
        Line::new(self.start.coord(), self.end.coord())
    }
}

/// Like a LineString but with Nodes
pub struct NodeString<N>(pub Vec<N>);

impl<'a, N: Clone> NodeString<N> {
    pub fn iter(&'a self) -> impl Iterator<Item = &'a N> {
        self.0.iter()
    }

    pub fn to_lines(&self) -> Vec<NodeLine<N>> {
        self.iter()
            .zip(self.iter().skip(1))
            .map(|(o, d)| NodeLine::new(o.clone(), d.clone()))
            .collect::<Vec<_>>()
    }
}


/// Like a Polygon but with NodeStrings
pub struct NodePolygon<N> {
    exterior: NodeString<N>,
    interiors: Vec<NodeString<N>>
}

impl<'a, N: Clone> NodePolygon<N> {
    pub fn nodes_iter(&'a self) -> impl Iterator<Item = &'a N> {
        self.exterior
            .iter()
            .chain(self.interiors.iter().map(|ns| ns.iter()).flatten())
    }
}


/// Trait for Node id field
pub trait IdType {}


impl<I, T> From<NodePolygon<Node<I, T>>> for VisibilityGraph<Node<I, T>> 
where
    I: IdType,
    T: CoordinateType + Float + std::fmt::Debug,
    Line<T>: Intersects<Line<T>>,
    Node<I, T>: PartialEq + Clone
{
    fn from(node_poly: NodePolygon<Node<I, T>>) -> VisibilityGraph<Node<I, T>> {
        let mut graph = VisibilityGraph::new();
        let edges: Vec<_> = node_poly.interiors
            .iter()
            .map(|ns| ns.to_lines())
            .flatten()
            .collect();
        for node in node_poly.nodes_iter() {
            let other_vertices: Vec<_> = node_poly.nodes_iter()
                .filter(|&n| *n != *node)
                .cloned()
                .collect();
            let visible_coords = rotational_sweep(node.clone(),
                                                  &other_vertices,
                                                  &edges);
            for target in visible_coords {
                graph.edge_list.push((node.clone(), target.clone()));
            }
        }
        graph
    }
}
