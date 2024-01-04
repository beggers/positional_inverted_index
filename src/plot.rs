use plotters::prelude::*;
use std::{
    error::Error,
    path::Path
};

pub fn plot_indexing_duration(target_dir: &str) -> Result<(), Box<dyn Error>> {
    let input_path = Path::new(target_dir).join("indexing_data.csv");
    let output_path = Path::new(target_dir).join("indexing_data.png");

    let mut rdr = csv::Reader::from_path(input_path)?;
    let mut data = Vec::new();
    let mut max_indexing_duration = 0u128;
    let mut max_documents = 0i32;

    for result in rdr.records() {
        let record = result?;
        let document_count: i32 = record[0].parse().expect("Failed to read document count!");
        let indexing_duration: u128 = record[1].parse().unwrap();

        data.push((document_count, indexing_duration));
        max_indexing_duration = max_indexing_duration.max(indexing_duration);
        max_documents = document_count;
    }

    let y_axis_upper_bound = max_indexing_duration;

    let root = BitMapBackend::new(&output_path, (1280, 960)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Document Count vs Indexing Duration", ("sans-serif", 20))
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0..max_documents, 0u128..y_axis_upper_bound)?;

    chart.configure_mesh().draw()?;
    chart.draw_series(LineSeries::new(data, &RED))?;

    Ok(())
}
