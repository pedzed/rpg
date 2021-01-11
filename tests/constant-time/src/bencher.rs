use super::Cdf;

pub struct Bencher {
    pub measurements: Vec<u64>,
}

impl Bencher {
    pub fn new(measurement_count: usize) -> Self {
        Self {
            measurements: Vec::with_capacity(measurement_count),
        }
    }

    pub fn calc_cdf(&mut self) -> Cdf {
        self.measurements.sort();
        let sum: u128 = self.measurements.iter().sum::<u64>().into();

        let mut cummulative_value = 0.0;

        self.measurements
            .iter()
            .map(|&measurement| {
                let distribution = (measurement as f64) / (sum as f64);
                cummulative_value += distribution;
                (measurement as f32, cummulative_value as f32)
            })
            .collect::<Cdf>()
    }
}
