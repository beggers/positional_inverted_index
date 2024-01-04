use plotters::prelude::*;
use std::{
    error::Error,
    f64,
    path::Path
};

pub fn plot_indexing_duration(target_dir: &str) -> Result<(), Box<dyn Error>> {
    let input_path = Path::new(target_dir).join("indexing_data.csv");
    let output_path = Path::new(target_dir).join("indexing_data.png");

    let mut rdr = csv::Reader::from_path(input_path)?;
    let (data, max_indexing_duration) = rdr
        .records()
        .filter_map(Result::ok)
        .fold((Vec::new(), 0f64), |(mut data, max_duration), record| {
            let document_count: i32 = record[0].parse().expect("Failed to read document count!");
            let indexing_duration: f64 = record[1].parse().unwrap();

            data.push((document_count, indexing_duration));
            (data, max_duration.max(indexing_duration))
        });

    let y_axis_upper_bound = max_indexing_duration * 1.5;

    let root = BitMapBackend::new(&output_path, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Document Count vs Indexing Duration", ("sans-serif", 20))
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0..1000, 0f64..y_axis_upper_bound)?;

    chart.configure_mesh().draw()?;
    chart.draw_series(LineSeries::new(data, &RED))?;

    Ok(())
}
