use std::ops::Add;
use std::ops::Sub;

use serde_derive::Deserialize;
use serde_derive::Serialize;
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sub() {
        let point = Point { x: 1, y: 1 };
        let other_point = Point { x: -1, y: -1 };
        let origin = Point { x: 0, y: 0 };
        assert_eq!(origin, point - point);
        assert_eq!(other_point, origin - point);
        assert_eq!(point, point - origin);
    }

    #[test]
    fn test_add() {
        let point = Point { x: 1, y: 1 };
        let other_point = Point { x: 2, y: 2 };
        let origin = Point { x: 0, y: 0 };
        assert_eq!(other_point, point + point);
        assert_eq!(point, origin + point);
        assert_eq!(point, point + origin);
    }

    #[test]
    fn test_eq() {
        let point = Point { x: 1, y: 1 };
        let origin = Point { x: 0, y: 0 };
        assert_eq!(true, point == point);
        assert_eq!(false, origin == point);
        assert_eq!(true, origin == origin);
    }
}
