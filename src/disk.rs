use crate::Point;
use plotters::prelude::*;
use std::f64::consts::PI;

fn disk_image(change_list: &[i32], out_file: &str, dimension: &str) {
    let file = format!("{}_{}.png", out_file, dimension);
    let root = BitMapBackend::new(&file, (5000, 5000)).into_drawing_area();

    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .build_cartesian_2d(-5000.0..5000.0, -5000.0..5000.0)
        .unwrap();
    let section = 360.0 / (change_list.len()) as f64;
    let mut changes = 0;
    let point_list = change_list
        .iter()
        .enumerate()
        .map(|(i, current_point)| {
            let next_point = change_list
                .get(i + 1)
                .unwrap_or_else(|| change_list.get(i).unwrap());
            let change = current_point - next_point;
            changes += change;
            let angle = f64::from(i as i32) * section;
            let pi_angle = (PI * 2.0 * angle) / 360.0;
            let radius = f64::from(changes as i32 + 4000);
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

pub fn make_disks(points: &[Point], dimensions: &Point, out_file: &str) {
    let middle_x = dimensions.x / 2;
    let middle_y = dimensions.y / 2;
    let changes_x = points
        .iter()
        //.step_by(10)
        .map(|point| point.x - middle_x)
        .collect::<Vec<_>>();
    disk_image(&changes_x, out_file, "x");
    let changes_y = points
        .iter()
        .map(|point| middle_y - point.y)
        .collect::<Vec<_>>();

    disk_image(&changes_y, out_file, "y");
}