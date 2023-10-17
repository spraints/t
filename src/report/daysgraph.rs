use std::path::Path;

//use plotters::prelude::{BitMapBackend, ChartBuilder};
//use plotters::style::IntoFont;

use super::days::Report;

pub fn plot(outfile: impl AsRef<Path> + std::fmt::Display, report: Report) {
    // let root = BitMapBackend::new(&outfile, (1200, 800)).into_drawing_area();
    // //root.fill(&WHITE);
    // let mut chart = ChartBuilder::on(&root)
    //     .caption("minutes worked per day", ("sans-serif", 50).into_font())
    //     .margin(5)
    //     .x_label_area_size(30)
    //     .y_label_area_size(30)
    //     .build_cartesian_2d(todo!(), todo!());
    todo!("{} {}", outfile, report.years.len())
}
