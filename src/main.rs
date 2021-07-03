use clap::{App, Arg};
use image::GenericImageView;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::fs;

mod disk;
mod plotting;
mod point;
mod tracing;
use disk::make_disks;
use plotting::{display_image, save_image};
use point::Point;
use tracing::generator_vec;
fn main() {
    let matches = argument_parse();
    let input = matches
        .value_of("INPUT")
        .expect("input file should always be present");

    let output = matches
        .value_of("OUTPUT")
        .expect("output file should always be present");

    let verbose = matches.occurrences_of("v");

    if matches.subcommand_matches("vector_dump").is_some() {
        let img = image::open(input).expect("could not open file");
        let points = generator_vec(&img, verbose);
        if verbose == 1 {
            dbg!(&points);
        }
        if verbose == 4 {
            display_image(&points, img.dimensions(), "plotters-doc-data/final.png").unwrap();
        }

        let (img_x, img_y) = img.dimensions();
        let center = Point {
            x: img_x as i32,
            y: img_y as i32,
        };
        let file_struct = PointsFile {
            points,
            dimensions: center,
        };
        let toml = toml::to_string(&file_struct).unwrap();
        fs::write(output, toml).expect("Unable to write toml file");
    }

    if matches.subcommand_matches("debug_image").is_some() {
        let toml = fs::read_to_string(input).expect("Unable to read file");
        let points_file: PointsFile = toml::from_str(&toml).unwrap();
        save_image(&points_file.points, &points_file.dimensions, output).unwrap();
    }

    if matches.subcommand_matches("disk_images").is_some() {
        let toml = fs::read_to_string(input).expect("Unable to read file");
        let points_file: PointsFile = toml::from_str(&toml).unwrap();
        make_disks(&points_file.points, &points_file.dimensions, output);
    }

    if matches.subcommand_matches("process_file").is_some() {
        let img = image::open(input).expect("could not open file");
        let points = generator_vec(&img, verbose);
        let (img_x, img_y) = img.dimensions();
        let center = Point {
            x: img_x as i32,
            y: img_y as i32,
        };
        make_disks(&points, &center, output);
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct PointsFile {
    points: Vec<Point>,
    dimensions: Point,
}

fn argument_parse() -> clap::ArgMatches {
    App::new("drawlings")
        .version("1.0")
        .author("becker")
        .arg(
            Arg::new("INPUT")
                .about("input file location")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("OUTPUT")
                .about("output file location")
                .required(true)
                .index(2),
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
                .about("dump vectors from image into toml file")
                .version("0.0"),
        )
        .subcommand(
            App::new("debug_image")
                .about("takes toml points and creates png")
                .version("0.0"),
        )
        .subcommand(
            App::new("disk_images")
                .about("takes toml points and disk pngs output file should not include extension")
                .version("0.0"),
        )
        .subcommand(
            App::new("process_file")
                .about("chains all commands together and outputs only disk images")
                .version("0.0"),
        )
        .get_matches()
}
