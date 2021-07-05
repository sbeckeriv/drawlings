use crate::plotting::save_image;
use crate::point::Point;
use image::{DynamicImage, GenericImageView};
// Ideas:
// Process lines not pixels. run the length of X until its white.. then process over x process each
// section of y taking the middle of the "line". how does this work around curves? I think this
// works. a long line could be writing as point 1 0,0 point 2 0,100 if its the same thickness.
// lets say in the middle it gets wide..it would 0,0   5,50  0,100 so like the line bumps a little
// in the middle.
//
// post process the vec to smooth and reduce duplicates or small artifacts. like of it went right
// then down.. remove the right..
//
//process line in both directions why not. once that is figured out..sort lines in two directions
//from two spots using threads.

// every direction around a point
const RIGHT: Point = Point { x: 1, y: 0 };
const RIGHT_TOP: Point = Point { x: 1, y: 1 };
const RIGHT_BOTTOM: Point = Point { x: 1, y: -1 };

const LEFT: Point = Point { x: -1, y: 0 };
const LEFT_TOP: Point = Point { x: -1, y: 1 };
const LEFT_BOTTOM: Point = Point { x: -1, y: -1 };

const TOP: Point = Point { x: 0, y: 1 };
const BOTTOM: Point = Point { x: 0, y: -1 };
const CENTER: Point = Point { x: 0, y: 0 };

const WHITE_PIXEL: image::Rgba<u8> = image::Rgba([255, 255, 255, 255]);
const BLACK_PIXEL: image::Rgba<u8> = image::Rgba([0, 0, 0, 255]);

// get the list of points along the line.
pub fn generator_vec(img: &DynamicImage, verbose: u64) -> Vec<Point> {
    // i think i want to create a velocity for better tracking of which way to go..

    // allow the points to be this many pixels away. 1 keeps it simple but more could make sense.
    // I like removing points on the output of the disk images to keep this math simpler.
    // Do not visit these again
    let mut bad_points = vec![];

    let mut return_points = vec![];
    // for the first point use this.
    let default_direction = vec![
        RIGHT,
        RIGHT_TOP,
        TOP,
        RIGHT_BOTTOM,
        BOTTOM,
        LEFT_BOTTOM,
        LEFT_TOP,
        LEFT,
    ];

    let mut direction_change = default_direction.clone();

    let first_spot = first_spot(img);
    if verbose == 1 {
        dbg!(&first_spot);
    }
    let mut current_spot = first_spot.expect("No non-white pixels located");
    let first_spot = first_spot.unwrap();
    let mut run = 0;
    return_points.push(current_spot);

    'main_loop: loop {
        run += 1;
        let mut next = None;
        if verbose == 1 {
            dbg!(return_points.len());
        }
        'top_loop: for direct in &direction_change {
            let point = *direct + current_spot;

            if return_points.contains(&point) || bad_points.contains(&point) {
                if verbose == 1 {
                    dbg!(
                        &point,
                        (return_points.contains(&point), bad_points.contains(&point))
                    );
                }
                continue;
            }

            let pixel = img.get_pixel(point.x as u32, point.y as u32);
            if pixel != WHITE_PIXEL {
                next = Some(point);
                break 'top_loop;
            } else if verbose == 1 {
                dbg!(point, pixel);
            }
        }

        match next {
            None => {
                if verbose == 1 {
                    dbg!(&current_spot, "before pop");
                }
                // roll back the path if we couldnt go around this point.
                let bad_parent = return_points.pop().expect("No parents could not find path");
                bad_points.push(bad_parent);
                current_spot = return_points
                    .last()
                    .expect("No parents could not find path")
                    .clone();
                if verbose == 1 {
                    dbg!(&current_spot, "after pop");
                }
            }

            Some(next) => {
                current_spot = next;
                return_points.push(current_spot);
                // dont check until we are far from the start
                if return_points.len() > 100 && at_the_start(&current_spot, &first_spot)
                    || return_points.len() > 100_000
                {
                    break 'main_loop;
                }
            }
        }

        if verbose == 4 && (run % 100 == 0) {
            save_image(&return_points, img.dimensions(), &format!("{}.png", run)).unwrap();
        }

        // Use the last two points to get the preferred next direction.
        let last_two = return_points.iter().rev().take(2).collect::<Vec<_>>();
        direction_change = if last_two.len() == 2 {
            next_directions(last_two.get(0).unwrap(), last_two.get(1).unwrap())
        } else {
            default_direction.clone()
        };
    }
    return_points
}
// Keep going in the same direction if we can.
fn next_directions(from: &Point, to: &Point) -> Vec<Point> {
    let point = *to - *from;
    match point {
        CENTER => {
            vec![
                RIGHT,
                RIGHT_TOP,
                TOP,
                RIGHT_BOTTOM,
                BOTTOM,
                LEFT_BOTTOM,
                LEFT_TOP,
                LEFT,
            ]
        }
        RIGHT => {
            vec![
                RIGHT,
                RIGHT_TOP,
                RIGHT_BOTTOM,
                TOP,
                BOTTOM,
                LEFT_TOP,
                LEFT_BOTTOM,
                LEFT,
            ]
        }
        RIGHT_TOP => {
            vec![
                RIGHT_TOP,
                RIGHT,
                TOP,
                RIGHT_BOTTOM,
                LEFT_TOP,
                BOTTOM,
                LEFT,
                LEFT_BOTTOM,
            ]
        }
        RIGHT_BOTTOM => {
            vec![
                RIGHT_BOTTOM,
                RIGHT,
                BOTTOM,
                RIGHT_TOP,
                LEFT_BOTTOM,
                LEFT,
                TOP,
                LEFT_TOP,
            ]
        }
        LEFT => {
            vec![
                LEFT,
                LEFT_TOP,
                LEFT_BOTTOM,
                TOP,
                BOTTOM,
                RIGHT_TOP,
                RIGHT_BOTTOM,
                RIGHT,
            ]
        }
        LEFT_TOP => {
            vec![
                LEFT_TOP,
                TOP,
                LEFT,
                RIGHT_TOP,
                LEFT_BOTTOM,
                RIGHT,
                BOTTOM,
                RIGHT_BOTTOM,
            ]
        }
        LEFT_BOTTOM => {
            vec![
                LEFT_BOTTOM,
                BOTTOM,
                LEFT,
                LEFT_TOP,
                RIGHT_BOTTOM,
                RIGHT,
                TOP,
                RIGHT_TOP,
            ]
        }
        TOP => {
            vec![
                TOP,
                RIGHT_TOP,
                LEFT_TOP,
                LEFT,
                RIGHT,
                LEFT_BOTTOM,
                RIGHT_BOTTOM,
                BOTTOM,
            ]
        }
        BOTTOM => {
            vec![
                BOTTOM,
                LEFT_BOTTOM,
                RIGHT_BOTTOM,
                LEFT,
                RIGHT,
                LEFT_TOP,
                RIGHT_TOP,
                TOP,
            ]
        }
        _ => None.unwrap_or_else(|| {
            panic!(
                "Can not get directions from this point collection {:?} from:{:?} to:{:?}",
                point, from, to
            )
        }),
    }
}

// Just get me any first spot. I thought it would work from the center
// but this was easier to reason about..
fn first_spot(img: &DynamicImage) -> Option<Point> {
    let (img_x, img_y) = img.dimensions();

    for move_x in 0..img_x {
        for move_y in 0..img_y {
            let pixel = img.get_pixel(move_x, move_y);
            if pixel != WHITE_PIXEL {
                return Some(Point {
                    x: move_x as i32,
                    y: move_y as i32,
                });
            }
        }
    }
    None
}

// let there be some buffer if we are back to the start.
fn at_the_start(current: &Point, start: &Point) -> bool {
    let point = *current - *start;
    point.x.abs() < 10 && point.y.abs() < 5
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::*;
    #[test]
    fn test_first_spot() {
        let blank = image::open("test/images/blank.png").unwrap();
        let dot = image::open("test/images/dot.png").unwrap();
        assert_eq!(None, first_spot(&blank));
        assert_eq!(Some(Point { x: 0, y: 0 }), first_spot(&dot));
    }

    #[test]
    fn test_at_the_start() {
        let start = Point { x: 0, y: 0 };
        let current = Point { x: 11, y: 6 };
        assert!(!at_the_start(&current, &start));

        let start = Point { x: 0, y: 0 };
        let current = Point { x: 9, y: 6 };
        assert!(!at_the_start(&current, &start));

        let start = Point { x: 0, y: 0 };
        let current = Point { x: 9, y: 4 };
        assert!(at_the_start(&current, &start));

        let start = Point { x: 0, y: 0 };
        let current = Point { x: -9, y: 4 };
        assert!(at_the_start(&current, &start));

        let start = Point { x: 0, y: 0 };
        let current = Point { x: 9, y: -4 };
        assert!(at_the_start(&current, &start));
    }
}
