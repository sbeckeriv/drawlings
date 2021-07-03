use crate::point::Point;
use plotters::prelude::*;

pub fn save_image(
    points: &[Point],
    size: &Point,
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    display_image(points, (size.x as u32, size.y as u32), output_file)
}
pub fn display_image(
    points: &[Point],
    size: (u32, u32),
    out_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(&out_file, size).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart =
        ChartBuilder::on(&root).build_cartesian_2d(0..size.0 as i32, 0..size.1 as i32)?;

    let point_list = points
        .iter()
        .map(|p| (p.x, size.1 as i32 - p.y))
        .collect::<Vec<_>>();

    let style = plotters::style::ShapeStyle {
        color: BLACK.to_rgba(),
        filled: false,
        stroke_width: 1,
    };
    chart.draw_series(std::iter::once(PathElement::new(point_list, style)))?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file");
    Ok(())
}
