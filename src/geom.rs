/// Geometry primatives for the sulu suite

use std::{
    str::FromStr,
    convert::TryInto,
};
use osmpbfreader::objects::Node;
use crate::error::Error;


#[derive(Debug, PartialEq, Clone)]
pub struct BBox {
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
    pub top: f64
}

pub struct Point {
    pub x: f64,
    pub y: f64
}

impl BBox {
    pub fn contains(&self, p: &Point) -> bool {
        (p.x >= self.left) && (p.x <= self.right) &&
            (p.y >= self.bottom) && (p.y <= self.top)
    }
}

impl From<(f64, f64)> for Point {
    fn from(tup: (f64, f64)) -> Point {
        Point{ x: tup.0, y: tup.1 }
    }
}

impl From<&Node> for Point {
    fn from(n: &Node) -> Point {
        (n.lon(), n.lat()).into()
    }
}

impl From<(f64, f64, f64, f64)> for BBox {
    fn from(tup: (f64, f64, f64, f64)) -> BBox {
        BBox {
            left: tup.0,
            bottom: tup.1,
            right: tup.2,
            top: tup.3
        }
    }
}

impl From<BBox> for [f64; 4] {
    fn from(b: BBox) -> [f64; 4] {
        [b.left, b.bottom, b.right, b.top]
    }
}

impl std::convert::TryFrom<Vec<f64>> for BBox {
    type Error = Error;

    fn try_from(v: Vec<f64>) -> Result<Self, Self::Error> {
        if v.len() == 4 {
            return Ok(BBox {
                left: v[0],
                bottom: v[1],
                right: v[2],
                top: v[3]
            })
        } else {
            return Err(Error::ConversionError(format!("Failed to convert {:?} to BBox - must have length 4", v)))
        }
    }
}

impl FromStr for BBox {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(",")
            .map(|s| s.parse::<f64>()
                 .map_err(|e| Error::ConversionError(e.to_string())))
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
    }
}

#[cfg(feature="sulu-http")]
use rocket::{
    request::{FromFormValue, FromParam},
    http::{RawStr}
};

#[cfg(feature="sulu-http")]
impl<'v> FromFormValue<'v> for BBox {
    type Error = Error;

    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        BBox::from_str(form_value.as_str())
    }
}

#[cfg(feature="sulu-http")]
impl<'r> FromParam<'r> for BBox {
    type Error = Error;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        BBox::from_str(param.as_str())
    }
}

