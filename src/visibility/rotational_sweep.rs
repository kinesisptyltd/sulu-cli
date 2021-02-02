#![allow(dead_code)]
use num_traits::float::Float;
use geo::{
    Coordinate,
    CoordinateType,
    Line,
    algorithm::intersects::Intersects
};

type CoordTuple<T> = (Coordinate<T>, Coordinate<T>);


fn rotational_sweep<T>(
    v: Coordinate<T>,
    other_vertices: &Vec<Coordinate<T>>,
    edges: &Vec<Line<T>>
) -> Vec<Coordinate<T>>
where
    T: CoordinateType + Float + std::fmt::Debug,
    Line<T>: Intersects<Line<T>>
{
    let h: Coordinate<_> = [T::one(), T::zero()].into();
    let mut queue: Vec<Coordinate<T>> = other_vertices.iter()
        .filter(|&x| angle(*x - v, h).is_finite())
        .cloned()
        .collect();
    
    // NB: queue has filtered out any NaN or inf - unwrap is ok
    queue.sort_by(|&x1, &x2| angle(x1 - v, h).partial_cmp(&angle(x2 - v, h)).unwrap());
    let mut seeing: Vec<Line<T>> = edges.iter()
        .filter(|e| (e.start.y - v.y) * (v.y - e.end.y) >= T::zero()) // e crosses the horizontal subtended from v
        .cloned()
        .collect();

    let mut visible_coords = vec![];

    for x in queue {
        let line = Line::new(v, x);
        if !seeing.iter()
            .filter(|e| (e.start != x) && (e.end != x))
            .any(|e| line.intersects(e)) {
            // x is visible from v
            visible_coords.push(x.clone());
        }
        let edges_with_x: Vec<Line<T>> = edges.iter()
            .filter(|e| (e.start == x) || (e.end == x))
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
        let edges = vec![]; // no obstacles
        let visible_points = rotational_sweep(v, &other_v, &edges);
        let expected = vec![
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
        let edges = vec![
            [(0.25, 0.25), (0.25, 0.75)].into(),
            [(0.25, 0.75), (0.75, 0.75)].into(),
            [(0.75, 0.75), (0.75, 0.25)].into(),
            [(0.75, 0.25), (0.25, 0.25)].into()]; // The inner square obstacle
        let visible_points = rotational_sweep(v, &other_v, &edges);
        let expected = vec![
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
        let edges = vec![[(0.5, 0.5), (0.5, f64::infinity())].into()]; 
        let visible_points = rotational_sweep(v, &other_v, &edges);
        let expected = vec![(1.0, 0.0).into()];
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
        let edges = vec![[(0.5, 0.5), (0.5, f64::nan())].into()]; 
        let visible_points = rotational_sweep(v, &other_v, &edges);
        let expected = vec![(1.0, 0.0).into()];
        visible_points.into_iter()
            .zip(expected.into_iter())
            .for_each(|(e, exp)| assert_eq!(e, exp));
    }
}
