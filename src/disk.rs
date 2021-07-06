use crate::Point;
use plotters::prelude::*;
use std::f64::consts::PI;

// Does the math to turn the line of distant to the center to a circle.
// plots them on the graph.
fn disk_image(change_list: &[i32], out_file: &str, dimension: &str, final_size: (u32, u32)) {
    let file = format!("{}_{}.png", out_file, dimension);
    let root = BitMapBackend::new(&file, final_size).into_drawing_area();

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
        .map(|(i, current_point)| points_to_radius(i, current_point, section))
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

// Rewrite Point to support floats to return a point here..
fn points_to_radius(i: usize, current_point: &i32, section: f64) -> (f64, f64) {
    let angle = f64::from(i as i32) * section;
    let pi_angle = (PI * 2.0 * angle) / 360.0;
    let radius = f64::from(*current_point + 4000);
    (radius * (pi_angle).cos(), radius * (pi_angle).sin())
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
    disk_image(&changes_x, out_file, "x", (4000, 4000));

    let changes_y = points
        .iter()
        .step_by(step_by)
        .map(|point| point.y - middle_y)
        .collect::<Vec<_>>();
    disk_image(&changes_y, out_file, "y", (4000, 4000));
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use float_cmp::approx_eq;
    #[test]
    fn test_points_to_radius() {
        let point = points_to_radius(0, &-3999, 1.0);
        assert!(approx_eq!(f64, 1.0, point.0, ulps = 2));
        assert!(approx_eq!(f64, 0.0, point.1, ulps = 2));

        let point = points_to_radius(90, &-3999, 1.0);
        dbg!(point);
        assert!(approx_eq!(f64, 0.0, point.0, epsilon = 0.00000003));
        assert!(approx_eq!(f64, 1.0, point.1, ulps = 2));

        let point = points_to_radius(180, &-3999, 1.0);
        assert!(approx_eq!(f64, -1.0, point.0, ulps = 2));
        assert!(approx_eq!(f64, 0.0, point.1, epsilon = 0.00000003));

        let point = points_to_radius(270, &-3999, 1.0);
        assert!(approx_eq!(f64, 0.0, point.0, epsilon = 0.00000003));
        assert!(approx_eq!(f64, -1.0, point.1, ulps = 2));
    }

    #[test]
    fn test_disk_image() {
        let out_file = "test/images/disk";
        let points = vec![
            7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14, 13, 14, 14, 15, 15, 16, 15, 16,
            16, 17, 17, 16, 17, 16, 17, 16, 17, 16, 17, 16, 17, 16, 15, 15, 14, 15, 14, 14, 13, 13,
            12, 13, 12, 11, 11, 10, 10, 9, 9, 8, 8, 7, 7, 6, 7, 6, 6, 5, 5, 4, 5, 4, 4,
        ];
        // doesnt work with other circles
        disk_image(&points, out_file, "x", (40, 40))
    }
}
