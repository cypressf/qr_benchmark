use anyhow::Result;
use plotters::prelude::*;
use std::collections::HashMap;
use std::fs::File;

#[derive(serde::Deserialize)]
struct Record {
    library: String,
    category: String,
    // file_path: String,
    // iteration: u32,
    duration_us: u64,
    status: String,
    // expected_text: String,
    // decoded_text: String,
}

pub fn generate_plots(csv_path: &str) -> Result<()> {
    let file = File::open(csv_path)?;
    let mut rdr = csv::Reader::from_reader(file);

    let mut stats: HashMap<(String, String), (u32, u32)> = HashMap::new(); // (Lib, Cat) -> (Correct, Total)
    let mut durations: HashMap<String, Vec<u64>> = HashMap::new(); // Lib -> Vec<Duration> (only correct ones)

    let mut libraries = std::collections::HashSet::new();
    let mut categories = std::collections::HashSet::new();

    for result in rdr.deserialize() {
        let record: Record = result?;

        libraries.insert(record.library.clone());
        categories.insert(record.category.clone());

        let key = (record.library.clone(), record.category.clone());
        let entry = stats.entry(key).or_insert((0, 0));
        entry.1 += 1;

        if record.status == "Correct" {
            entry.0 += 1;
            durations
                .entry(record.library.clone())
                .or_default()
                .push(record.duration_us);
        }
    }

    let mut sorted_libraries: Vec<String> = libraries.into_iter().collect();
    sorted_libraries.sort();
    let mut sorted_categories: Vec<String> = categories.into_iter().collect();
    sorted_categories.sort();

    // 1. Success Rate Plot
    draw_success_rates(&sorted_categories, &sorted_libraries, &stats)?;

    // 2. Performance Plot (Summary)
    draw_performance(&sorted_libraries, &durations)?;

    // 3. Performance Distribution (Histogram/PDF)
    draw_performance_dist(&sorted_libraries, &durations)?;

    Ok(())
}

fn draw_performance_dist(libraries: &[String], durations: &HashMap<String, Vec<u64>>) -> Result<()> {
    let root = BitMapBackend::new("performance_dist.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut all_durations = Vec::new();
    for list in durations.values() {
        all_durations.extend(list.iter().cloned());
    }
    
    if all_durations.is_empty() {
        return Ok(()); // Nothing to draw
    }

    all_durations.sort();
    // Clip outliers for better visualization (e.g., P98)
    let max_dur = all_durations[(all_durations.len() as f64 * 0.98) as usize];
    
    let bucket_count = 50;
    let bucket_size = (max_dur as f64 / bucket_count as f64).ceil() as u64;
    let bucket_size = bucket_size.max(1); // avoid 0

    let mut chart = ChartBuilder::on(&root)
        .caption("Performance Distribution (PDF)", ("sans-serif", 40))
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(0u64..(max_dur + bucket_size), 0.0..1.0)?; // Normalized frequency

    chart
        .configure_mesh()
        .x_desc("Duration (us)")
        .y_desc("Density")
        .draw()?;

    for (idx, lib) in libraries.iter().enumerate() {
        if let Some(durs) = durations.get(lib) {
            let color = Palette99::pick(idx);
            
            // Build histogram
            let mut buckets = HashMap::new();
            for &d in durs {
                if d <= max_dur {
                    let b = d / bucket_size;
                    *buckets.entry(b).or_insert(0) += 1;
                }
            }
            
            let total = durs.len() as f64;
            let mut points = Vec::new();
            
            for b in 0..=bucket_count {
                 let count = *buckets.get(&(b as u64)).unwrap_or(&0);
                 let density = count as f64 / total;
                 points.push((b as u64 * bucket_size, density));
            }
            // Add end point to drop line
            points.push(((bucket_count as u64 + 1) * bucket_size, 0.0));

            chart
                .draw_series(LineSeries::new(
                    points,
                    &color,
                ))?
                .label(lib)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2)));
                
             // Optional: Fill area with low opacity
             /*
             chart.draw_series(AreaSeries::new(
                points,
                0.0,
                &color.mix(0.2),
             ))?;
             */
        }
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}


fn draw_success_rates(
    categories: &[String],
    libraries: &[String],
    stats: &HashMap<(String, String), (u32, u32)>,
) -> Result<()> {
    let root = BitMapBackend::new("success_rates.png", (1280, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let x_max = categories.len() as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption("Success Rate by Category", ("sans-serif", 40))
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..x_max, 0.0..1.05)?;

    chart
        .configure_mesh()
        .x_labels(categories.len())
        .x_label_formatter(&|x| {
            let idx = x.floor() as usize;
            if idx < categories.len() {
                categories[idx].clone()
            } else {
                "".to_string()
            }
        })
        .y_label_formatter(&|y| format!("{:.0}%", y * 100.0))
        .draw()?;

    let bar_width = 0.8 / libraries.len() as f64;

    for (lib_idx, lib) in libraries.iter().enumerate() {
        let color = Palette99::pick(lib_idx);

        let mut bars = Vec::new();

        for cat_idx in 0..categories.len() {
            let cat = &categories[cat_idx];
            let (correct, total) = stats.get(&(lib.clone(), cat.clone())).unwrap_or(&(0, 1));
            let rate = *correct as f64 / *total as f64;

            let center = cat_idx as f64 + 0.5;
            let offset = (lib_idx as f64 - (libraries.len() - 1) as f64 / 2.0) * bar_width;
            let x0 = center + offset - bar_width / 2.0;
            let x1 = center + offset + bar_width / 2.0;

            if rate > 0.0 {
                bars.push(Rectangle::new([(x0, 0.0), (x1, rate)], color.filled()));
            }
        }

        chart
            .draw_series(bars)?
            .label(lib)
            .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled()));
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}

fn draw_performance(libraries: &[String], durations: &HashMap<String, Vec<u64>>) -> Result<()> {
    let root = BitMapBackend::new("performance.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    // Calculate stats
    let mut perf_stats = Vec::new();
    let mut max_dur = 0;

    for lib in libraries {
        if let Some(durs) = durations.get(lib) {
            let mut sorted = durs.clone();
            sorted.sort();
            let median = sorted[sorted.len() / 2];
            let p95 = sorted[(sorted.len() as f64 * 0.95) as usize];
            max_dur = max_dur.max(p95);
            perf_stats.push((lib, median, p95));
        } else {
            perf_stats.push((lib, 0, 0));
        }
    }

    // Ensure non-zero range
    if max_dur == 0 {
        max_dur = 100;
    }

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Performance (Median Duration - Correct Decodes)",
            ("sans-serif", 30),
        )
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(
            0.0..libraries.len() as f64,
            0u64..(max_dur as u64 + (max_dur / 10).max(10)),
        )?;

    chart
        .configure_mesh()
        .x_labels(libraries.len())
        .x_label_formatter(&|x| {
            let idx = x.floor() as usize;
            if idx < libraries.len() {
                libraries[idx].clone()
            } else {
                "".to_string()
            }
        })
        .y_desc("Microseconds")
        .draw()?;

    let bar_width = 0.6;

    for (idx, (_lib, median, _p95)) in perf_stats.iter().enumerate() {
        let color = Palette99::pick(idx);
        let center = idx as f64 + 0.5;
        let x0 = center - bar_width / 2.0;
        let x1 = center + bar_width / 2.0;
        
        if *median > 0 {
            chart.draw_series(std::iter::once(Rectangle::new(
                [
                    (x0, 0),
                    (x1, *median),
                ],
                color.filled(),
            )))?;
        }
    }

    Ok(())
}
