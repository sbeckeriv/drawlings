use crate::disk::points_to_radius;
use crate::plotting::save_image;
use crate::point::Point;
use image::{DynamicImage, GenericImageView};
use std::collections::HashSet;
use std::iter::FromIterator;
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
const DEFAULT_DIRECTIONS: [Point; 8] = [
    RIGHT,
    RIGHT_TOP,
    TOP,
    RIGHT_BOTTOM,
    BOTTOM,
    LEFT_BOTTOM,
    LEFT_TOP,
    LEFT,
];

const WHITE_PIXEL: image::Rgba<u8> = image::Rgba([255, 255, 255, 255]);

fn black_pixel(pixel: &image::Rgba<u8>) -> bool {
    pixel[0] < 255 && pixel[0] == pixel[1] && pixel[1] == pixel[2]
}

fn search_for_other_line(
    origin_point: &Point,
    pixel: &image::Rgba<u8>,
    img: &DynamicImage,
    excluded_points: HashSet<Point>,
) -> Option<Point> {
    let angles: Vec<usize> = (0..360).step_by(10).collect();
    let mut next_point = None;
    let mut distance = 2;
    loop {
        let mut found = angles
            .iter()
            .map(|angle| {
                let point = points_to_radius(*angle, distance, 1.0);
                Point {
                    x: point.0 as i32,
                    y: point.1 as i32,
                } + *origin_point
            })
            .collect::<Vec<Point>>();
        found.dedup();
        let mut found = found.iter().filter(|point| {
            excluded_points.get(point).is_none()
                && img.in_bounds(point.x as u32, point.y as u32)
                && img.get_pixel(point.x as u32, point.y as u32) == *pixel
        });
        if let Some(point) = found.next() {
            next_point = Some(*point);
            break;
        } else if distance > img.dimensions().0 as i32 && distance > img.dimensions().1 as i32 {
            break;
        } else {
            distance += 1;
        }
    }
    next_point
}

fn get_line_points(
    origin_point: &Point,
    pixel: &image::Rgba<u8>,
    img: &DynamicImage,
) -> HashSet<Point> {
    let mut points = HashSet::new();
    let mut seen_neighbors = HashSet::new();
    let mut neighbors = vec![*origin_point];

    while let Some(next_point) = neighbors.pop() {
        for direction in DEFAULT_DIRECTIONS.iter() {
            let moved = *direction + next_point;
            let moved_pixel = img.get_pixel(moved.x as u32, moved.y as u32);

            if seen_neighbors.get(&next_point).is_none() && moved_pixel == *pixel {
                neighbors.push(moved);
            } else if black_pixel(&moved_pixel) {
                points.insert(moved);
            }
        }
        seen_neighbors.insert(next_point);
    }
    points
}

fn get_points_of_color(
    origin_point: &Point,
    pixel: &image::Rgba<u8>,
    img: &DynamicImage,
) -> HashSet<Point> {
    let mut points = HashSet::new();
    let mut neighbors = vec![*origin_point];
    while let Some(next_point) = neighbors.pop() {
        for direction in DEFAULT_DIRECTIONS.iter() {
            let moved = *direction + next_point;
            if points.get(&next_point).is_none()
                && img.get_pixel(moved.x as u32, moved.y as u32) == *pixel
            {
                neighbors.push(moved);
            }
        }
        points.insert(next_point);
    }
    points
}

fn locate_other_edge(point: Point, pixel: image::Rgba<u8>, img: &DynamicImage) -> Option<Point> {
    // get a hash set of all points in the current line. to exclude in our search
    let start_line_points = get_points_of_color(&point, &pixel, img);
    // search from the current point in a circle pattern for another color point not in the hash
    // set
    dbg!(&start_line_points);
    if let Some(next_color_line_point) =
        search_for_other_line(&point, &pixel, img, start_line_points)
    {
        dbg!(&next_color_line_point);
        // take that blob and find all the black pixels that touch it. This is the other side of the line
        let next_line_points = get_line_points(&next_color_line_point, &pixel, img);

        dbg!(&next_line_points);
        let mut next_points = Vec::from_iter(next_line_points.iter());
        next_points.sort();
        // get fancy and find a "center" point. Honestly i didnt confirm sort works as i want but
        // its a basic idea.
        next_points
            .get(next_points.len() / 2).map(|point| *point.to_owned())
    } else {
        None
    }
    //
}

// get the list of points along the line.
pub fn generator_vec(img: &DynamicImage, verbose: u64) -> Vec<Point> {
    // i think i want to create a velocity for better tracking of which way to go..

    // allow the points to be this many pixels away. 1 keeps it simple but more could make sense.
    // I like removing points on the output of the disk images to keep this math simpler.
    // Do not visit these again
    let mut bad_points = vec![];

    let mut return_points = vec![];

    let mut direction_change = DEFAULT_DIRECTIONS.clone().to_vec();
    let mut used_cross_over_colors = vec![];

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

        if verbose == 1 {
            dbg!(&current_spot, "before direct");
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

            if verbose == 1 {
                dbg!(&point, &pixel);
            }
            if pixel != WHITE_PIXEL && !used_cross_over_colors.contains(&pixel) {
                if black_pixel(&pixel) {
                    next = Some(point);
                } else {
                    if verbose >= 1 {
                        dbg!("color pixel found", &pixel);
                    }
                    next = locate_other_edge(point, pixel, img);
                    dbg!(&next);
                    used_cross_over_colors.push(pixel);
                }
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
                current_spot = *return_points
                    .last()
                    .expect("No parents could not find path");

                if verbose == 1 {
                    dbg!(&current_spot, "after pop");
                }
            }

            Some(next) => {
                current_spot = next;
                return_points.push(current_spot);
                // dont check until we are far from the start
                if return_points.len() > 50 && at_the_start(&current_spot, &first_spot)
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
            // keep the last direction if none. we should panic if there was no color gap
            next_directions(last_two.get(0).unwrap(), last_two.get(1).unwrap())
                .unwrap_or(direction_change)
        } else {
            DEFAULT_DIRECTIONS.clone().to_vec()
        };
    }
    return_points
}

// Keep going in the same direction if we can.
fn next_directions(from: &Point, to: &Point) -> Option<Vec<Point>> {
    let point = *to - *from;
    let direction = match point {
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
        _ => vec![],
    };

    if !direction.is_empty() {
        Some(direction)
    } else {
        None
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

    #[test]
    fn test_first_spot() {
        let blank = image::open("test/images/blank.png").unwrap();
        let dot = image::open("test/images/dot.png").unwrap();
        assert_eq!(None, first_spot(&blank));
        assert_eq!(Some(Point { x: 0, y: 0 }), first_spot(&dot));
    }

    macro_rules! direction_test {
        ($direction:ident) => {
            #[test]
            fn $direction() {
                let from = Point { x: 0, y: 0 };
                let to = super::$direction.clone();
                let list = next_directions(&from, &to).unwrap();
                assert_eq!(*list.first().expect("should be a list"), super::$direction);
            }
        };
    }

    direction_test!(TOP);
    direction_test!(RIGHT_TOP);
    direction_test!(LEFT_TOP);
    direction_test!(LEFT);
    direction_test!(RIGHT);
    direction_test!(LEFT_BOTTOM);
    direction_test!(RIGHT_BOTTOM);
    direction_test!(BOTTOM);

    #[test]
    fn test_next_direction_to_far() {
        let from = Point { x: 0, y: 0 };
        let to = Point { x: 0, y: 2 };
        assert_eq!(None, next_directions(&from, &to));
    }

    #[test]
    fn test_close_broken_circle() {
        let img = image::open("test/images/close_broken_circle.png").expect("could not open file");
        let points = generator_vec(&img, 4);
        assert_eq!(68, points.len())
    }

    #[test]
    #[should_panic]
    fn test_broken_circle() {
        let img = image::open("test/images/broken_circle.png").expect("could not open file");
        generator_vec(&img, 4);
    }

    #[test]
    fn test_full_circle() {
        let img = image::open("test/images/full_circle.png").expect("could not open file");
        let points = generator_vec(&img, 4);
        let t: Vec<Point> = vec![
            Point { x: 3, y: 7 },
            Point { x: 4, y: 7 },
            Point { x: 3, y: 8 },
            Point { x: 4, y: 8 },
            Point { x: 3, y: 9 },
            Point { x: 4, y: 9 },
            Point { x: 3, y: 10 },
            Point { x: 4, y: 10 },
            Point { x: 3, y: 11 },
            Point { x: 4, y: 11 },
            Point { x: 3, y: 12 },
            Point { x: 4, y: 12 },
            Point { x: 3, y: 13 },
            Point { x: 4, y: 13 },
            Point { x: 4, y: 14 },
            Point { x: 5, y: 13 },
            Point { x: 5, y: 14 },
            Point { x: 6, y: 14 },
            Point { x: 5, y: 15 },
            Point { x: 6, y: 15 },
            Point { x: 6, y: 16 },
            Point { x: 7, y: 15 },
            Point { x: 7, y: 16 },
            Point { x: 8, y: 16 },
            Point { x: 7, y: 17 },
            Point { x: 8, y: 17 },
            Point { x: 9, y: 16 },
            Point { x: 9, y: 17 },
            Point { x: 10, y: 16 },
            Point { x: 10, y: 17 },
            Point { x: 11, y: 16 },
            Point { x: 11, y: 17 },
            Point { x: 12, y: 16 },
            Point { x: 12, y: 17 },
            Point { x: 13, y: 16 },
            Point { x: 13, y: 17 },
            Point { x: 14, y: 16 },
            Point { x: 13, y: 15 },
            Point { x: 14, y: 15 },
            Point { x: 14, y: 14 },
            Point { x: 15, y: 15 },
            Point { x: 15, y: 14 },
            Point { x: 16, y: 14 },
            Point { x: 15, y: 13 },
            Point { x: 16, y: 13 },
            Point { x: 16, y: 12 },
            Point { x: 17, y: 13 },
            Point { x: 17, y: 12 },
            Point { x: 16, y: 11 },
            Point { x: 17, y: 11 },
            Point { x: 16, y: 10 },
            Point { x: 17, y: 10 },
            Point { x: 16, y: 9 },
            Point { x: 17, y: 9 },
            Point { x: 16, y: 8 },
            Point { x: 17, y: 8 },
            Point { x: 16, y: 7 },
            Point { x: 17, y: 7 },
            Point { x: 16, y: 6 },
            Point { x: 15, y: 7 },
            Point { x: 15, y: 6 },
            Point { x: 14, y: 6 },
            Point { x: 15, y: 5 },
            Point { x: 14, y: 5 },
            Point { x: 14, y: 4 },
            Point { x: 13, y: 5 },
            Point { x: 13, y: 4 },
            Point { x: 12, y: 4 },
        ];
        assert_eq!(t, points);
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
