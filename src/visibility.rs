#![allow(dead_code)]
use num_traits::real::Real;
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
) -> Vec<CoordTuple<T>>
where
    T: CoordinateType + Real + PartialOrd + std::fmt::Debug,
    Line<T>: Intersects<Line<T>>
{
    let h: Coordinate<_> = [T::one(), T::zero()].into();
    let mut angles: Vec<_> = other_vertices.iter()
        .map(|&x| (angle(x - v, h), x.clone()))
        .collect();
    angles.sort_by(|(a1, _), (a2, _)| a1.partial_cmp(a2).unwrap());
    let mut seeing: Vec<Line<T>> = edges.iter()
        .filter(|e| (e.start.y - v.y) * (v.y - e.end.y) >= T::zero()) // e crosses the horizontal subtended from v
        .cloned()
        .collect();

    let mut new_edges = vec![];

    for (_, x)  in angles {
        let line = Line::new(v, x);
        if !seeing.iter().any(|e| line.intersects(e)) {
            // x is visible from v
            new_edges.push((v, x.clone()));
        }
        let edges_with_x: Vec<Line<T>> = edges.iter()
            .filter(|e| (e.start == x) || (e.end == x))
            .cloned()
            .collect();
        let remove_these: Vec<Line<T>> = seeing.iter()
            .filter(|&s| edges_with_x.iter().any(|&e| *s == e))
            .cloned()
            .collect();
        let mut add_these: Vec<Line<T>> = edges_with_x.into_iter()
            .filter(|e| seeing.iter().any(|s| e != s))
            .collect();
        seeing = seeing.into_iter()
            .filter(|s| remove_these.iter().any(|e| s != e))
            .collect();
        seeing.append(&mut add_these)
    }
    new_edges
}

fn angle<T: CoordinateType + Real>(u: Coordinate<T>, v: Coordinate<T>) -> T {
    (dot(u, v) / (norm(u) * norm(v))).acos()
}

fn dot<T: CoordinateType>(u: Coordinate<T>, v: Coordinate<T>) -> T {
    u.x * v.x + u.y * v.y
}

fn norm<T: CoordinateType + Real>(v: Coordinate<T>) -> T {
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
        let new_edges = rotational_sweep(v, &other_v, &edges);
        let expected = vec![
            ((0.0, 0.0).into(), (1.0, 0.0).into()),
            ((0.0, 0.0).into(), (1.0, 1.0).into()),
            ((0.0, 0.0).into(), (0.0, 1.0).into())];
        new_edges.into_iter()
            .zip(expected.into_iter())
            .for_each(|(e, exp)| assert_eq!(e, exp));
    }
}
