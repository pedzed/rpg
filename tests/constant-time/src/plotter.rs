use std::path::Path;

use plotters::prelude::*;

use super::Cdf;

pub struct Plotter {
    pub chart_title: String,
    pub chart_x_spec: std::ops::Range<f32>,
    pub data_fixed: Cdf,
    pub data_random: Cdf,
    pub image_path: Box<Path>,
}

impl Plotter {
    pub fn plot(&self) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(&self.image_path, (640, 480)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(&self.chart_title, ("sans-serif", 22).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(
                self.chart_x_spec.to_owned(),
                0f32..1.1f32,
            )?
        ;

        chart.configure_mesh().draw()?;

        chart
            .draw_series(LineSeries::new(
                self.data_fixed.to_owned(),
                &RED,
            ))?
            .label("Fixed")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED))
        ;

        chart
            .draw_series(LineSeries::new(
                self.data_random.to_owned(),
                &BLUE,
            ))?
            .label("Random")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE))
        ;

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?
        ;

        Ok(())
    }
}
