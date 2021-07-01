use clap::{App, Arg, ArgMatches};
use image::{DynamicImage, GenericImageView, Pixel};
use plotters::prelude::*;
use std::ops::Add;

const OUT_FILE_NAME: &str = "plotters-doc-data/snowflake.png";
fn main() {
    let matches = argument_parse();
    let input = matches
        .value_of("INPUT")
        .expect("input file should always be present");

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    let verbose = matches.occurrences_of("v");

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level app
    if let Some(_matches) = matches.subcommand_matches("vector_dump") {
        let img = image::open(input).expect("could not open file");
        let points = generator_vec(&img, verbose);

        dbg!(points.len());
        display_image(&points, img.dimensions(), "final".into()).unwrap();
    }
}

// add maths. add sub.. move all tuples to points
// i know there is more then one lib for this.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
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
//
//
const RIGHT: Point = Point { x: 1, y: 0 };
const RIGHT_TOP: Point = Point { x: 1, y: 1 };
const RIGHT_BOTTOM: Point = Point { x: 1, y: -1 };

const LEFT: Point = Point { x: -1, y: 0 };
const LEFT_TOP: Point = Point { x: -1, y: 1 };
const LEFT_BOTTOM: Point = Point { x: -1, y: -1 };

const TOP: Point = Point { x: 0, y: 1 };
const BOTTOM: Point = Point { x: 0, y: -1 };

const WHITE_PIXEL: image::Rgba<u8> = image::Rgba([255, 255, 255, 255]);

// get the list of points along the line.
fn generator_vec(img: &DynamicImage, verbose: u64) -> Vec<Point> {
    // i think i want to create a velocity for better tracking of which way to go..

    // allow the points to be this many pixels away.
    let pixel_movement = 1;
    let mut bad_points = vec![];

    let mut return_points = vec![];
    // for the first point use this.
    let mut direction_change = vec![
        RIGHT,
        RIGHT_TOP,
        TOP,
        RIGHT_BOTTOM,
        BOTTOM,
        LEFT_BOTTOM,
        LEFT_TOP,
        LEFT,
    ];

    let (img_x, img_y) = img.dimensions();

    // off by 1 for odd size images
    let _middle = Point {
        x: (img_x / 2) as i32,
        y: (img_y / 2) as i32,
    };

    dbg!("here");
    let first_spot = first_spot(img);

    dbg!(&first_spot);
    let mut current_spot = first_spot.expect("No non-white pixels located");
    let first_spot = first_spot.unwrap();
    let mut run = 0;
    return_points.push(current_spot);
    'main_loop: loop {
        run += 1;
        let mut next = None;
        if verbose > 1 {
            dbg!(return_points.len());
        }
        'top_loop: for direct in &direction_change {
            let move_x = direct.x + current_spot.x;
            let move_y = direct.y + current_spot.y;
            let point = Point {
                x: move_x * pixel_movement,
                y: move_y * pixel_movement,
            };

            if return_points.contains(&point) || bad_points.contains(&point) {
                if verbose > 1 {
                    dbg!(
                        &point,
                        (return_points.contains(&point), bad_points.contains(&point))
                    );
                }
                continue;
            }

            let pixel = img.get_pixel(move_x as u32, move_y as u32);
            if pixel != WHITE_PIXEL {
                next = Some(point);
                break 'top_loop;
            } else {
                if verbose > 1 {
                    dbg!(point, pixel);
                }
            }
        }
        match next {
            None => {
                if verbose > 1 {
                    dbg!(&current_spot, "before pop");
                }
                let bad_parent = return_points.pop().expect("No parents could not find path");
                bad_points.push(bad_parent);
                current_spot = return_points.pop().expect("No parents could not find path");
                if verbose > 1 {
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
        if false && (run > 4400 && run < 4650) {
            //if run % 100 == 0 {
            display_image(&&return_points, img.dimensions(), format!("{}", run)).unwrap();
        }
        let last_two = return_points.iter().rev().take(2).collect::<Vec<_>>();
        direction_change = if last_two.len() == 2 {
            next_directions(
                last_two.get(0).unwrap(),
                last_two.get(1).unwrap(),
                pixel_movement,
            )
        } else {
            vec![
                RIGHT,
                RIGHT_BOTTOM,
                RIGHT_TOP,
                BOTTOM,
                TOP,
                LEFT_BOTTOM,
                LEFT_TOP,
                LEFT,
            ]
        };
    }
    return_points
}

fn next_directions(from: &Point, to: &Point, pixel_movement: i32) -> Vec<Point> {
    let x = (to.x - from.x) / pixel_movement;
    let y = (to.y - from.y) / pixel_movement;
    let point = Point { x: x, y: y };
    //dbg!(from, to, &point);
    match point {
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
    }
}
// Just get me any first spot. I thought it would work from the center
// but this was easier to reason about..
fn first_spot(img: &DynamicImage) -> Option<Point> {
    let (img_x, img_y) = img.dimensions();

    let mut first_spot = None;
    'top_loop: while first_spot.is_none() {
        for move_x in 0..img_x {
            for move_y in 0..img_y {
                let pixel = img.get_pixel(move_x, move_y);
                if pixel != WHITE_PIXEL {
                    first_spot = Some(Point {
                        x: move_x as i32,
                        y: move_y as i32,
                    });
                    break 'top_loop;
                }
            }
        }
    }
    first_spot
}
// let there be some buffer if we are back to the start.
fn at_the_start(current: &Point, start: &Point) -> bool {
    (current.x - start.x).abs() < 10 && (current.y - start.y).abs() < 5
}

fn argument_parse() -> clap::ArgMatches {
    App::new("drawlings")
        .version("1.0")
        .author("becker")
        .arg(
            Arg::new("INPUT")
                .about("input file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("v")
                .short('v')
                .multiple_occurrences(true)
                .takes_value(true)
                .about("Sets the level of verbosity"),
        )
        .subcommand(
            App::new("vector_dump")
                .about("dump vectors from image")
                .version("0.0"),
        )
        .get_matches()
}

fn display_image(
    points: &Vec<Point>,
    size: (u32, u32),
    file_apend: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let out_file = format!("plotters-doc-data/snowflake_{}.png", file_apend);
    let root = BitMapBackend::new(&out_file, size).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        //.caption("Koch's Snowflake", ("sans-serif", 50))
        .build_cartesian_2d(0..size.0 as i32, 0..size.1 as i32)?;

    let mut snowflake_vertices = points
        .iter()
        .map(|p| (p.x, size.1 as i32 - p.y))
        .collect::<Vec<_>>();

    chart.draw_series(std::iter::once(PathElement::new(
        snowflake_vertices,
        &BLACK,
    )))?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", out_file);
    Ok(())
}
