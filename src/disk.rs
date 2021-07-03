use crate::Point;
use plotters::prelude::*;
use std::f64::consts::PI;

// Does the math to turn the line of distant to the center to a circle.
// plots them on the graph.
fn disk_image(change_list: &[i32], out_file: &str, dimension: &str) {
    let file = format!("{}_{}.png", out_file, dimension);
    let root = BitMapBackend::new(&file, (5000, 5000)).into_drawing_area();

    root.fill(&WHITE).unwrap();
    root.draw(&Circle::new(
        (2500, 2500),
        50,
        Into::<ShapeStyle>::into(&GREEN).filled(),
    ))
    .unwrap();

    root.draw(&Circle::new(
        (2500, 2500),
        2000,
        Into::<ShapeStyle>::into(&RED).stroke_width(1),
    ))
    .unwrap();

    let mut chart = ChartBuilder::on(&root)
        .build_cartesian_2d(-5000.0..5000.0, -5000.0..5000.0)
        .unwrap();

    // if there are 5000 points use them all!!
    // to get the x,y on a circle it is angle(cos for x sin for y) * radius.
    // angle is pi*2 (tau) * angle / 360
    // radius is base + the distance from the center.
    let section = 360.0 / (change_list.len()) as f64;
    let point_list = change_list
        .iter()
        .enumerate()
        .map(|(i, current_point)| {
            let angle = f64::from(i as i32) * section;
            let pi_angle = (PI * 2.0 * angle) / 360.0;
            let radius = f64::from(*current_point as i32 + 4000);
            (radius * (pi_angle).cos(), radius * (pi_angle).sin())
        })
        .collect::<Vec<_>>();

    let style = plotters::style::ShapeStyle {
        color: BLACK.to_rgba(),
        filled: false,
        stroke_width: 1,
    };
    chart
        .draw_series(std::iter::once(PathElement::new(point_list, style)))
        .unwrap();

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file");
}

// Converts the points to circles.
// Output should be file location without type. /path/to/dir/butterfly_
pub fn make_disks(points: &[Point], dimensions: &Point, out_file: &str) {
    let step_by = 1; // make configurable
    let middle_x = dimensions.x / 2;
    let middle_y = dimensions.y / 2;
    let changes_x = points
        .iter()
        .step_by(step_by)
        .map(|point| point.x - middle_x)
        .collect::<Vec<_>>();
    disk_image(&changes_x, out_file, "x");

    let changes_y = points
        .iter()
        .step_by(step_by)
        .map(|point| point.y - middle_y)
        .collect::<Vec<_>>();
    disk_image(&changes_y, out_file, "y");
}
