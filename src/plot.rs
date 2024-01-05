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

    if all_data.len() > 10_000 {
        let (means, uppers, lowers, max_indexing_duration) = group_and_process_data(&all_data, 1000); // Adjust the number of groups here
        plot_line_with_std_dev(
            &means,
            &uppers,
            &lowers,
            &output_path,
            means.len() as i32,
            max_indexing_duration as f64,
            "Document Count vs Index Latency (µs)",
            "Document Count",
            "Index Latency (µs)",
        )?;
    } else {
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
    }

    Ok(())
}

fn group_and_process_data(data: &Vec<(i32, u128)>, num_groups: usize)
    -> (Vec<(i32, f64)>, Vec<(i32, f64)>, Vec<(i32, f64)>, u128) {

    let group_size = data.len() / num_groups;
    let mut mean_data = Vec::new();
    let mut std_dev_upper_data = Vec::new();
    let mut std_dev_lower_data = Vec::new();
    let mut max_indexing_duration = 0u128;

    for chunk in data.chunks(group_size) {
        let (sum, sum_squares, count) = chunk.iter().fold((0u128, 0u128, 0usize), |(sum, sum_squares, count), &(_, duration)| {
            (sum + duration, sum_squares + duration.pow(2), count + 1)
        });

        let mean = sum as f64 / count as f64;
        let std_dev = ((sum_squares as f64 / count as f64) - (mean.powi(2))).sqrt();
        let document_count = chunk.first().unwrap().0;
        
        mean_data.push((document_count, mean));
        std_dev_upper_data.push((document_count, mean + std_dev));
        // Ensure that the lower bound of standard deviation is not less than 0
        std_dev_lower_data.push((document_count, (mean - std_dev).max(0.0)));
        max_indexing_duration = max_indexing_duration.max(sum / count as u128);
    }

    (mean_data, std_dev_upper_data, std_dev_lower_data, max_indexing_duration)
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

<<<<<<< HEAD
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
        "Document Count vs Posting List Size",
        "Document Count",
        "Size",
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

=======
>>>>>>> parent of 7ca681f (Posting list sizes graphing)
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
