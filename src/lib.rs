// Copyright 2014-2015 The GeoRust Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(unstable)]

use std::ascii::AsciiExt;

use tokenizer::{PeekableTokens, Token, Tokens};
use types::FromTokens;
use types::geometrycollection::GeometryCollection;
use types::linestring::LineString;
use types::point::Point;
use types::polygon::Polygon;
use types::multipoint::MultiPoint;
use types::multilinestring::MultiLineString;
use types::multipolygon::MultiPolygon;

mod tokenizer;
mod types;


pub enum WktItem {
    Point(Point),
    LineString(LineString),
    Polygon(Polygon),
    MultiPoint(MultiPoint),
    MultiLineString(MultiLineString),
    MultiPolygon(MultiPolygon),
    GeometryCollection(GeometryCollection),
}

impl WktItem {
    fn from_word_and_tokens(word: &str, tokens: &mut PeekableTokens)-> Result<Self, &'static str> {
        match word {
            "POINT" => {
                let x = <Point as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            },
            "LINESTRING" => {
                let x = <LineString as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            },
            "POLYGON" => {
                let x = <Polygon as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            },
            "MULTIPOINT" => {
                let x = <MultiPoint as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            },
            "MULTILINESTRING" => {
                let x = <MultiLineString as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            },
            "MULTIPOLYGON" => {
                let x = <MultiPolygon as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            },
            "GEOMETRYCOLLECTION" => {
                let x = <GeometryCollection as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            },
            _ => Err("Invalid type encountered"),
        }
    }
}


pub struct Wkt {
    items: Vec<WktItem>
}

impl Wkt {
    fn new() -> Self {
        Wkt {items: vec![]}
    }

    fn add_item(&mut self, item: WktItem) {
        self.items.push(item);
    }

    fn from_str(wkt_str: &str) -> Result<Self, &'static str> {
        let tokens = Tokens::from_str(wkt_str);
        Wkt::from_tokens(tokens)
    }

    fn from_tokens(tokens: Tokens) -> Result<Self, &'static str> {
        let mut wkt = Wkt::new();
        let mut tokens = tokens.peekable();
        let word = match tokens.next() {
            Some(Token::Word(word)) => {
                if !word.is_ascii() {
                    return Err("Encountered non-ascii word");
                }
                word.to_ascii_uppercase()
            },
            None => return Ok(wkt),
            _ => return Err("Invalid WKT format"),
        };
        match WktItem::from_word_and_tokens(word.as_slice(), &mut tokens) {
            Ok(item) => wkt.add_item(item),
            Err(s) => return Err(s),
        }
        Ok(wkt)
    }
}


#[cfg(test)]
mod tests {
    use super::{Wkt, WktItem};

    #[test]
    fn empty_string() {
        let wkt = Wkt::from_str("").ok().unwrap();
        assert_eq!(0, wkt.items.len());
    }

    #[test]
    fn basic_point() {
        let mut wkt = Wkt::from_str("POINT (10 -20)").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        let point = match wkt.items.pop().unwrap() {
            WktItem::Point(point) => point,
            _ => unreachable!(),
        };
        assert_eq!(10.0, point.coord.x);
        assert_eq!(-20.0, point.coord.y);
        assert_eq!(None, point.coord.z);
        assert_eq!(None, point.coord.m);
    }

    #[test]
    fn basic_point_whitespace() {
        let mut wkt = Wkt::from_str(" \n\t\rPOINT \n\t\r( \n\r\t10 \n\t\r-20 \n\t\r) \n\t\r").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        let point = match wkt.items.pop().unwrap() {
            WktItem::Point(point) => point,
            _ => unreachable!(),
        };
        assert_eq!(10.0, point.coord.x);
        assert_eq!(-20.0, point.coord.y);
        assert_eq!(None, point.coord.z);
        assert_eq!(None, point.coord.m);
    }

    #[test]
    fn basic_linestring() {
        let mut wkt = Wkt::from_str("LINESTRING (10 -20, -0 -0.5)").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        let linestring = match wkt.items.pop().unwrap() {
            WktItem::LineString(linestring) => linestring,
            _ => unreachable!(),
        };
        assert_eq!(2, linestring.coords.len());

        assert_eq!(10.0, linestring.coords[0].x);
        assert_eq!(-20.0, linestring.coords[0].y);
        assert_eq!(None, linestring.coords[0].z);
        assert_eq!(None, linestring.coords[0].m);

        assert_eq!(0.0, linestring.coords[1].x);
        assert_eq!(-0.5, linestring.coords[1].y);
        assert_eq!(None, linestring.coords[1].z);
        assert_eq!(None, linestring.coords[1].m);
    }

    #[test]
    fn basic_polygon() {
        let mut wkt = Wkt::from_str("POLYGON ((8 4, 4 0, 0 4, 8 4), (7 3, 4 1, 1 4, 7 3))").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        let linestring = match wkt.items.pop().unwrap() {
            WktItem::Polygon(linestring) => linestring,
            _ => unreachable!(),
        };
        assert_eq!(2, linestring.lines.len());
    }

    #[test]
    fn basic_multipoint() {
        let mut wkt = Wkt::from_str("MULTIPOINT ((8 4), (4 0))").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        let multipoint = match wkt.items.pop().unwrap() {
            WktItem::MultiPoint(multipoint) => multipoint,
            _ => unreachable!(),
        };
        assert_eq!(2, multipoint.points.len());
    }

    #[test]
    fn basic_multilinestring() {
        let mut wkt = Wkt::from_str("MULTILINESTRING ((8 4, -3 0), (4 0, 6 -10))").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        let multilinestring = match wkt.items.pop().unwrap() {
            WktItem::MultiLineString(multilinestring) => multilinestring,
            _ => unreachable!(),
        };
        assert_eq!(2, multilinestring.lines.len());
    }

    #[test]
    fn basic_multipolygon() {
        let mut wkt = Wkt::from_str("MULTIPOLYGON (((8 4)), ((4 0)))").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        let multipolygon = match wkt.items.pop().unwrap() {
            WktItem::MultiPolygon(multipolygon) => multipolygon,
            _ => unreachable!(),
        };
        assert_eq!(2, multipolygon.polygons.len());
    }

    #[test]
    fn basic_geometrycollection() {
        let mut wkt = Wkt::from_str("GEOMETRYCOLLECTION (POINT (8 4)))").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        let geometrycollection = match wkt.items.pop().unwrap() {
            WktItem::GeometryCollection(geometrycollection) => geometrycollection,
            _ => unreachable!(),
        };
        assert_eq!(1, geometrycollection.items.len());
    }

    #[test]
    fn invalid_points() {
        Wkt::from_str("POINT ()").err().unwrap();
        Wkt::from_str("POINT (10)").err().unwrap();
        Wkt::from_str("POINT 10").err().unwrap();
        Wkt::from_str("POINT (10 -20 40)").err().unwrap();
    }
}
