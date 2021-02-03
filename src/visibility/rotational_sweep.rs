#![allow(dead_code)]
use num_traits::float::Float;
use geo::{
    Coordinate,
    CoordinateType,
    Line,
    algorithm::intersects::Intersects
};

type CoordTuple<T> = (Coordinate<T>, Coordinate<T>);


pub trait CoordTrait<T: CoordinateType> {
    fn coord(&self) -> Coordinate<T>;
}

impl<T: CoordinateType> CoordTrait<T> for Coordinate<T> {
    fn coord(&self) -> Coordinate<T> {
        *self
    }
}

pub trait LineTrait<T: CoordinateType> {
    fn line(&self) -> Line<T>;
}

impl<T: CoordinateType> LineTrait<T> for Line<T> {
    fn line(&self) -> Line<T> {
        *self
    }
}


pub fn rotational_sweep<C, L, T>(
    v: C,
    other_vertices: &Vec<C>,
    edges: &Vec<L>
) -> Vec<C>
where
    C: CoordTrait<T> + Clone,
    T: CoordinateType + Float + std::fmt::Debug,
    L: LineTrait<T>,
    Line<T>: Intersects<Line<T>>
{
    let h: Coordinate<T> = [T::one(), T::zero()].into();
    let v0 = v.coord();
    let edges_lines: Vec<_> = edges.iter().map(|e| e.line()).collect();
    let mut queue: Vec<C> = other_vertices.iter()
        .filter(|&x| angle(x.coord() - v0, h).is_finite())
        .cloned()
        .collect();
    
    // NB: queue has filtered out any NaN or inf angles - unwrap is ok
    queue.sort_by(|x1, x2| angle(x1.coord() - v0, h).partial_cmp(&angle(x2.coord() - v0, h)).unwrap());
    let mut seeing: Vec<Line<T>> = edges_lines.iter()
        .filter(|e| (e.start.y - v0.y) * (v0.y - e.end.y) >= T::zero()) // e crosses the horizontal subtended from v
        .cloned()
        .collect();

    let mut visible_coords = vec![];

    for x in queue {
        let x0 = x.coord();
        let line = Line::new(v0, x0);
        if !seeing.iter()
            .filter(|e| (e.start != x0) && (e.end != x0))
            .any(|e| line.intersects(e)) {
            // x is visible from v
            visible_coords.push(x.clone());
        }
        let edges_with_x: Vec<Line<T>> = edges_lines.iter()
            .filter(|e| (e.start == x0) || (e.end == x0))
            .cloned()
            .collect();
        let remove_these: Vec<Line<T>> = seeing.iter()
            .filter(|s| contains(&edges_with_x, s))
            .cloned()
            .collect();
        let mut add_these: Vec<Line<T>> = edges_with_x.into_iter()
            .filter(|e| !contains(&seeing, e))
            .collect();
        seeing = seeing.into_iter()
            .filter(|s| !contains(&remove_these, s))
            .collect();
        seeing.append(&mut add_these);
    }
    visible_coords
}

fn contains<T: std::cmp::PartialEq>(v: &Vec<T>, elem: &T) -> bool {
    v.iter().any(|i| *i == *elem)
}

fn angle<T: CoordinateType + Float>(u: Coordinate<T>, v: Coordinate<T>) -> T {
    (dot(u, v) / (norm(u) * norm(v))).acos()
}

fn dot<T: CoordinateType>(u: Coordinate<T>, v: Coordinate<T>) -> T {
    u.x * v.x + u.y * v.y
}

fn norm<T: CoordinateType + Float>(v: Coordinate<T>) -> T {
    (v.x.powi(2) + v.y.powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_obstacles() {
        let v = (0.0, 0.0).into();
        let other_v = vec![
            (0.0, 1.0).into(),
            (1.0, 1.0).into(),
            (1.0, 0.0).into()]; // corners of a square
        let edges: Vec<Line<_>> = vec![]; // no obstacles
        let visible_points: Vec<Coordinate<_>> = rotational_sweep(v, &other_v, &edges);
        let expected: Vec<Coordinate<_>> = vec![
            (1.0, 0.0).into(),
            (1.0, 1.0).into(),
            (0.0, 1.0).into()];
        visible_points.into_iter()
            .zip(expected.into_iter())
            .for_each(|(e, exp)| assert_eq!(e, exp));
    }

    #[test]
    fn test_an_obstacle() {
        let v = (0.0, 0.0).into();
        let other_v = vec![
            (0.0, 1.0).into(), // |
            (1.0, 1.0).into(), // |
            (1.0, 0.0).into(), // -> Corners of the outer square
            (0.25, 0.25).into(),  // |   
            (0.25, 0.75).into(),  // |  
            (0.75, 0.75).into(),  // |  
            (0.75, 0.25).into()]; // -> Corners of the obstacle 
        let edges: Vec<Line<_>> = vec![
            [(0.25, 0.25), (0.25, 0.75)].into(),
            [(0.25, 0.75), (0.75, 0.75)].into(),
            [(0.75, 0.75), (0.75, 0.25)].into(),
            [(0.75, 0.25), (0.25, 0.25)].into()]; // The inner square obstacle
        let visible_points: Vec<Coordinate<_>> = rotational_sweep(v, &other_v, &edges);
        let expected: Vec<Coordinate<_>> = vec![
            (1.0, 0.0).into(),
            (0.75, 0.25).into(),
            (0.25, 0.25).into(),
            (0.25, 0.75).into(),
            (0.0, 1.0).into()];
        visible_points.into_iter()
            .zip(expected.into_iter())
            .for_each(|(e, exp)| assert_eq!(e, exp));
    }
    
    #[test]
    fn test_an_infinite_obstacle() {
        //
        // I don't really know what behaviour we should expect here
        //
        let v = (0.0, 0.0).into();
        let other_v = vec![
            (0.0, 1.0).into(), // |
            (1.0, 1.0).into(), // |
            (1.0, 0.0).into()]; // -> Corners of the outer square
        let edges: Vec<Line<_>> = vec![[(0.5, 0.5), (0.5, f64::infinity())].into()]; 
        let visible_points: Vec<Coordinate<_>> = rotational_sweep(v, &other_v, &edges);
        let expected: Vec<Coordinate<_>> = vec![(1.0, 0.0).into()];
        visible_points.into_iter()
            .zip(expected.into_iter())
            .for_each(|(e, exp)| assert_eq!(e, exp));
    }
    
    #[test]
    fn test_a_nan_obstacle() {
        //
        // I don't really know what behaviour we should expect here
        //
        let v = (0.0, 0.0).into();
        let other_v = vec![
            (0.0, 1.0).into(), // |
            (1.0, 1.0).into(), // |
            (1.0, 0.0).into()]; // -> Corners of the outer square
        let edges: Vec<Line<_>> = vec![[(0.5, 0.5), (0.5, f64::nan())].into()]; 
        let visible_points: Vec<Coordinate<_>> = rotational_sweep(v, &other_v, &edges);
        let expected: Vec<Coordinate<_>> = vec![(1.0, 0.0).into()];
        visible_points.into_iter()
            .zip(expected.into_iter())
            .for_each(|(e, exp)| assert_eq!(e, exp));
    }
}
