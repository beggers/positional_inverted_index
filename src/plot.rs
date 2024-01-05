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

    for result in rdr.records() {
        let record = result?;
        let document_count: i32 = record[0].parse().unwrap();
        let indexing_duration: u128 = record[1].parse().unwrap();

        data.push((document_count, indexing_duration));
        max_indexing_duration = max_indexing_duration.max(indexing_duration);
    }

    let y_axis_upper_bound = max_indexing_duration;

    plot_documents_to_latency_chart(
        data.clone(), 
        &output_path, 
        data.len() as i32,
        y_axis_upper_bound, 
        "Document Count vs Index Duration (µs)", 
        "Document Count", 
        "Index Duration (µs)",
    )?;
    Ok(())
}

pub fn plot_query_duration(target_dir: &str) -> Result<(), Box<dyn Error>> {
    let input_path = Path::new(target_dir).join("querying_data.csv");
    let output_path = Path::new(target_dir).join("querying_data.png");

    let mut rdr = csv::Reader::from_path(input_path)?;
    let mut data = Vec::new();
    let mut max_indexing_duration = 0u128;
    let mut max_query_tokens = 0usize;
    let mut x_axis_upper_bound = 0;

    for result in rdr.records() {
        let record = result?;
        let document_count: i32 = record[0].parse().unwrap();
        let query: String = record[1].parse().unwrap();
        let indexing_duration: u128 = record[2].parse().unwrap();

        data.push((document_count, indexing_duration));
        max_indexing_duration = max_indexing_duration.max(indexing_duration);
        max_query_tokens = max_query_tokens.max(query.split_whitespace().count());
        x_axis_upper_bound = x_axis_upper_bound.max(document_count);
    }

    let y_axis_upper_bound = max_indexing_duration;

    plot_documents_to_latency_chart(
        data.clone(), 
        &output_path, 
        x_axis_upper_bound,
        y_axis_upper_bound, 
        &("Document Count vs Query Duration (µs) - Max Query Tokens: ".to_owned() + &max_query_tokens.to_string()), 
        "Document Count", 
        "Query Duration (µs)",
    )?;
    Ok(())
}

fn plot_documents_to_latency_chart(
    data: Vec<(i32, u128)>, 
    output_path: &Path, 
    x_axis_upper_bound: i32,
    y_axis_upper_bound: u128, 
    title: &str,
    x_label: &str,
    y_label: &str,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(output_path, (1280, 960)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 20))
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(0..x_axis_upper_bound, 0u128..y_axis_upper_bound)?;

    chart.configure_mesh()
        .x_desc(x_label)
        .y_desc(y_label)
        .draw()?;

    chart.draw_series(LineSeries::new(data, &BLUE))?;
    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}
