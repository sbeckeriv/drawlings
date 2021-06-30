use clap::{App, Arg};
use image::{DynamicImage, GenericImageView, Pixel};
use plotters::prelude::*;

const OUT_FILE_NAME: &'static str = "plotters-doc-data/snowflake.png";
fn main() {
    let matches = App::new("drawlings")
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
        .get_matches();

    // You can check the value provided by positional arguments, or option arguments
    let input = matches
        .value_of("INPUT")
        .expect("input file should always be present");

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    let verbose = matches.occurrences_of("v");

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level app
    if let Some(ref matches) = matches.subcommand_matches("vector_dump") {
        let img = image::open("tests/images/jpg/progressive/cat.jpg").expect("could not open file");
        let vecs = generator_vec(&img);
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}
// get the list of points along the line.
fn generator_vec(img: &DynamicImage) -> Vec<Point> {
    // i think i want to create a velocity for better tracking of which way to go..

    // allow the points to be this many pixels away.
    let pixel_movement = 1;

    let mut return_points = vec![];

    let direction_change = vec![
        (pixel_movement, 0),
        (pixel_movement, pixel_movement),
        (-1 * pixel_movement, 0),
        (-1 * pixel_movement, -1 * pixel_movement),
    ];

    let black_pixel = image::Rgba([0, 0, 0, 0]);

    let (img_x, img_y) = img.dimensions();

    // off by 1 for odd size images
    let middle = Point {
        x: (img_x / 2) as i32,
        y: (img_y / 2) as i32,
    };

    let mut first_spot = first_spot(&img);

    let mut current_spot = first_spot.expect("no black pixels located");
    let first_spot = first_spot.unwrap();
    return_points.push(current_spot.clone());
    'main_loop: loop {
        let mut next = None;
        'top_loop: for direct in &direction_change {
            let move_x = direct.0 + current_spot.x;
            let move_y = direct.1 + current_spot.y;
            let point = Point {
                x: move_x,
                y: move_y,
            };
            if return_points.last().unwrap() == &point {
                next;
            }

            let pixel = img.get_pixel(move_x as u32, move_y as u32);
            if pixel == black_pixel {
                next = Some(point);
                break 'top_loop;
            }
        }
        current_spot = next.expect("No next black pixel found");
        return_points.push(current_spot.clone());
        // dont check until we are far from the start
        if return_points.len() > 10 && at_the_start(&current_spot, &first_spot) {
            break 'main_loop;
        }
    }

    return_points
}
// Just get me any first spot. I thought it would work from the center
// but this was easier to reason about..
fn first_spot(img: &DynamicImage) -> Option<Point> {
    let (img_x, img_y) = img.dimensions();
    let mut first_spot = None;
    let mut x = 0;
    let mut y = 0;
    let black_pixel = image::Rgba([0, 0, 0, 0]);
    'top_loop: while first_spot.is_none() {
        for move_x in 0..img_x {
            for move_y in 0..img_y {
                let pixel = img.get_pixel(move_x, move_y);
                if pixel == black_pixel {
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
    false
}

/*
fn display_image() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(OUT_FILE_NAME, (1024, 768)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        //.caption("Koch's Snowflake", ("sans-serif", 50))
        .build_cartesian_2d(-200.0..200.0, -200.0..200.0)?;

    let mut snowflake_vertices = {
        let mut current: Vec<(f64, f64)> = vec![
            (100.0, 100.0),
            (-100.0, -100.0),
            (-100.0, 100.0),
            (199.0, -100.0),
        ];
        current
    };

    chart.draw_series(std::iter::once(Polygon::new(
        snowflake_vertices.clone(),
        &TRANSPARENT.mix(0.2),
    )))?;
    snowflake_vertices.push(snowflake_vertices[0]);
    chart.draw_series(std::iter::once(PathElement::new(
        snowflake_vertices,
        &BLACK,
    )))?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);
    Ok(())
}

#[test]
fn entry_point() {
    main().unwrap()
}
*/
