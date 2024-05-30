// TODO fix someday
use plotters::prelude::*;

fn generate_graph(data: HashMap::<String, u32>) -> Result<(), Box<dyn std::error::Error>> {
    const OUT_FILE_NAME: &str = "histogram.png";
    let root = BitMapBackend::new(OUT_FILE_NAME, (640, 480)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .caption("Cdemonchy.com stats", ("sans-serif", 50.0))
        .build_cartesian_2d(0..(data.keys().len()-1) as u32, 0u32..*data.values().max().unwrap())?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc("Count")
        .x_desc("page")
        .x_label_formatter(&|x| format!("{:?}",data.clone().into_keys().collect::<Vec<String>>()[*x as usize]))
        .axis_desc_style(("sans-serif", 15))
        .draw()?;

    chart.draw_series(
        Histogram::vertical(&chart)
            .style(RED.mix(0.5).filled())
            .data(data.values().enumerate().map(|(x, y)| (x as u32, *y)))
            //.data(data.vqlues().map(|x: &u32| (*x, 1))),
    )?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);
    Ok(())
}