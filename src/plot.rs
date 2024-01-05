use plotters::prelude::*;
use std::{
    error::Error,
    path::Path
};

pub fn plot_index_latency(target_dir: &str) -> Result<(), Box<dyn Error>> {
    let input_path = Path::new(target_dir).join("index_latency.csv");
    let output_path = Path::new(target_dir).join("index_latency.png");

    let mut rdr = csv::Reader::from_path(input_path)?;
    let mut all_data = Vec::new();

    for result in rdr.records() {
        let record = result?;
        let document_count: i32 = record[0].parse().unwrap();
        let indexing_duration: u128 = record[1].parse().unwrap();
        all_data.push((document_count, indexing_duration));
    }

    let max_indexing_duration = all_data.iter().map(|&(_, y)| y).max().unwrap_or_default();
    plot_documents_to_latency_chart(
        all_data.clone(),
        &output_path,
        all_data.len() as i32,
        max_indexing_duration,
        "Document Count vs Index Latency (µs)",
        "Document Count",
        "Index Latency (µs)",
    )?;

    Ok(())
}

pub fn plot_query_latency(target_dir: &str) -> Result<(), Box<dyn Error>> {
    let input_path = Path::new(target_dir).join("query_latency.csv");
    let output_path = Path::new(target_dir).join("query_latency.png");

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
        &("Document Count vs Query Latency (µs) - Max Query Tokens: ".to_owned() + &max_query_tokens.to_string()), 
        "Document Count", 
        "Query Latency (µs)",
    )?;
    Ok(())
}

pub fn plot_term_list_sizes(target_dir: &str) -> Result<(), Box<dyn Error>> {
    let input_path = Path::new(target_dir).join("term_list_sizes.csv");
    let output_path = Path::new(target_dir).join("term_list_sizes.png");

    let mut rdr = csv::Reader::from_path(input_path)?;
    let mut data = Vec::new();
    let mut max_document_count = 0;
    let mut y_axis_upper_bound = 0;

    for result in rdr.records() {
        let record = result?;
        let document_count: i32 = record[0].parse().unwrap();
        let term_list_size: u128 = record[1].parse().unwrap();

        data.push((document_count, term_list_size));
        max_document_count = max_document_count.max(document_count);
        y_axis_upper_bound = y_axis_upper_bound.max(term_list_size);
    }

    plot_documents_to_latency_chart(
        data.clone(), 
        &output_path, 
        max_document_count,
        y_axis_upper_bound, 
        "Document Count vs Term List Size (bytes)",
        "Document Count",
        "Term List Size (bytes)",
    )?;
    Ok(())
}

pub fn plot_posting_list_distribution(target_dir: &str) -> Result<(), Box<dyn Error>> {
    let input_path = Path::new(target_dir).join("posting_list_sizes.csv");
    let output_path = Path::new(target_dir).join("posting_list_sizes.png");

    let mut rdr = csv::Reader::from_path(input_path)?;
    let mut mean_data = Vec::new();
    let mut std_dev_upper_data = Vec::new();
    let mut std_dev_lower_data = Vec::new();
    let mut max_document_count = 0;
    let mut y_axis_upper_bound: f64 = 0.0;

    for result in rdr.records() {
        let record = result?;
        let document_count: i32 = record[0].parse()?;
        let mean: f64 = record[1].parse()?;
        let std_dev: f64 = record[2].parse()?;

        mean_data.push((document_count, mean));
        std_dev_upper_data.push((document_count, mean + std_dev));
        std_dev_lower_data.push((document_count, (mean - std_dev).max(0.0)));
        max_document_count = max_document_count.max(document_count);
        y_axis_upper_bound = y_axis_upper_bound.max(mean + std_dev);
    }

    plot_line_with_std_dev(
        &mean_data,
        &std_dev_upper_data,
        &std_dev_lower_data,
        &output_path,
        max_document_count,
        y_axis_upper_bound,
        "Document Count vs Posting List Sizes (bytes)",
        "Document Count",
        "Posting List Sizes (bytes)",
    )?;
    Ok(())
}

// This is not the best way to do this but it works.
fn plot_line_with_std_dev(
    mean_data: &Vec<(i32, f64)>,
    std_dev_upper_data: &Vec<(i32, f64)>,
    std_dev_lower_data: &Vec<(i32, f64)>,
    output_path: &Path,
    x_axis_upper_bound: i32,
    y_axis_upper_bound: f64,
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
        .build_cartesian_2d(0..x_axis_upper_bound, 0.0f64..y_axis_upper_bound)?;

    chart.configure_mesh()
        .x_desc(x_label)
        .y_desc(y_label)
        .draw()?;

    chart.draw_series(LineSeries::new(mean_data.clone(), &BLUE))?
        .label("Mean Posting List Size")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    chart.draw_series(LineSeries::new(std_dev_upper_data.clone(), &GREEN))?
        .label("Mean +- Std Dev")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));
    chart.draw_series(LineSeries::new(std_dev_lower_data.clone(), &GREEN))?;

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

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
