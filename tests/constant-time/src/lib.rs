pub mod bencher;
pub mod plotter;

use bencher::Bencher;
use plotter::Plotter;

pub type Cdf = Vec<(f32, f32)>;

use std::io::Write;

use std::path::Path;

use aes_soft::cipher::*;

use aes::state::*;
use aes::round_key::RoundKey;

use tsc_time::{Start, Stop, Duration};
use rand::random;

pub fn measure_xor(bench_count: usize, plot_image_path: &Path) {
    println!("Started benchmark for xor.");
        println!("\tStarted M1: fixed measurements.");
            print!("\t\tSetting up Bencher...");
            let mut bencher = Bencher::new(bench_count);
            println!(" ✓");

            let num1: i32 = random();
            let num2: i32 = random();

            print!("\t\tRunning `{}` iterations...", bench_count); std::io::stdout().flush().unwrap();
            for _ in 0..bench_count {
                let start = Start::now();
                    let _xor = num1 ^ num2;
                let stop = Stop::now();

                let duration: Duration = stop - start;
                &bencher.measurements.push(duration.cycles().into());
            }
            println!(" ✓");

            print!("\t\tCalculating CDF values..."); std::io::stdout().flush().unwrap();
            let fixed_cdf = bencher.calc_cdf();
            println!(" ✓");
        println!("\tFinished M1: fixed measurements.");

        println!("\tStarted M2: random measurements.");
            print!("\t\tSetting up Bencher...");
            let mut bencher = Bencher::new(bench_count);
            println!(" ✓");

            print!("\t\tRunning `{}` iterations...", bench_count); std::io::stdout().flush().unwrap();
            for _ in 0..bench_count {
                let num1: i32 = random();
                let num2: i32 = random();

                let start = Start::now();
                    let _xor = num1 ^ num2;
                let stop = Stop::now();

                let duration: Duration = stop - start;
                &bencher.measurements.push(duration.cycles().into());
            }
            println!(" ✓");

            print!("\t\tCalculating CDF values..."); std::io::stdout().flush().unwrap();
            let random_cdf = bencher.calc_cdf();
            println!(" ✓");
        println!("\tFinished M2: random measurements.");

        print!("\tPlotting..."); std::io::stdout().flush().unwrap();
            Plotter {
                chart_title: format!("xor, {} iterations", bench_count),
                chart_x_spec: 60f32..90f32,
                data_fixed: fixed_cdf,
                data_random: random_cdf,
                image_path: plot_image_path.into(),
            }.plot().unwrap();
        println!(" ✓");
    println!("Finished benchmark for xor.");
}

pub fn measure_aes_soft(bench_count: usize, plot_image_path: &Path) {
    println!("Started benchmark for aes_soft.");
        println!("\tStarted M1: fixed measurements.");
            print!("\t\tSetting up Bencher...");
            let mut bencher = Bencher::new(bench_count);
            println!(" ✓");

            let key = generic_array::GenericArray::from_slice(&[0u8; 16]);
            let mut block = generic_array::GenericArray::clone_from_slice(&[0u8; 16]);
            let cipher = aes_soft::Aes128::new(&key);

            print!("\t\tRunning `{}` iterations...", bench_count); std::io::stdout().flush().unwrap();
            for _ in 0..bench_count {
                let start = Start::now();
                    cipher.encrypt_block(&mut block);
                let stop = Stop::now();

                let duration: Duration = stop - start;
                &bencher.measurements.push(duration.cycles().into());
            }
            println!(" ✓");

            print!("\t\tCalculating CDF values..."); std::io::stdout().flush().unwrap();
            let fixed_cdf = bencher.calc_cdf();
            println!(" ✓");
        println!("\tFinished M1: fixed measurements.");

        println!("\tStarted M2: random measurements.");
            print!("\t\tSetting up Bencher...");
            let mut bencher = Bencher::new(bench_count);
            println!(" ✓");

            print!("\t\tRunning `{}` iterations...", bench_count); std::io::stdout().flush().unwrap();
            for _ in 0..bench_count {
                let key = generic_array::GenericArray::from_slice(&[0u8; 16]);
                let mut block = generic_array::GenericArray::clone_from_slice(&[0u8; 16]);
                let cipher = aes_soft::Aes128::new(&key);

                let start = Start::now();
                    cipher.encrypt_block(&mut block);
                let stop = Stop::now();

                let duration: Duration = stop - start;
                &bencher.measurements.push(duration.cycles().into());
            }
            println!(" ✓");

            print!("\t\tCalculating CDF values..."); std::io::stdout().flush().unwrap();
            let random_cdf = bencher.calc_cdf();
            println!(" ✓");
        println!("\tFinished M2: random measurements.");

        print!("\tPlotting..."); std::io::stdout().flush().unwrap();
            Plotter {
                chart_title: format!("aes_soft, {} iterations", bench_count),
                chart_x_spec: 25000f32..60000f32,
                data_fixed: fixed_cdf,
                data_random: random_cdf,
                image_path: plot_image_path.into(),
            }.plot().unwrap();
        println!(" ✓");
    println!("Finished benchmark for aes_soft.");
}

pub fn measure_aes_add_round_key(bench_count: usize, plot_image_path: &Path) {
    println!("Started benchmark for aes_add_round_key.");
        println!("\tStarted M1: fixed measurements.");
            print!("\t\tSetting up Bencher...");
            let mut bencher = Bencher::new(bench_count);
            println!(" ✓");

            let key: RoundKey = random();
            let state = State::new(random());

            print!("\t\tRunning `{}` iterations...", bench_count); std::io::stdout().flush().unwrap();
            for _ in 0..bench_count {
                let mut state = state.clone();

                let start = Start::now();
                    state.add_round_key(key);
                let stop = Stop::now();

                let duration: Duration = stop - start;
                &bencher.measurements.push(duration.cycles().into());
            }
            println!(" ✓");

            print!("\t\tCalculating CDF values..."); std::io::stdout().flush().unwrap();
            let fixed_cdf = bencher.calc_cdf();
            println!(" ✓");
        println!("\tFinished M1: fixed measurements.");

        println!("\tStarted M2: random measurements.");
            print!("\t\tSetting up Bencher...");
            let mut bencher = Bencher::new(bench_count);
            println!(" ✓");

            print!("\t\tRunning `{}` iterations...", bench_count); std::io::stdout().flush().unwrap();
            for _ in 0..bench_count {
                let key: RoundKey = random();
                let mut state = State::new(random());

                let start = Start::now();
                    state.add_round_key(key);
                let stop = Stop::now();

                let duration: Duration = stop - start;
                &bencher.measurements.push(duration.cycles().into());
            }
            println!(" ✓");

            print!("\t\tCalculating CDF values..."); std::io::stdout().flush().unwrap();
            let random_cdf = bencher.calc_cdf();
            println!(" ✓");
        println!("\tFinished M2: random measurements.");

        print!("\tPlotting..."); std::io::stdout().flush().unwrap();
            Plotter {
                chart_title: format!("aes_add_round_key, {} iterations", bench_count),
                chart_x_spec: 90f32..180f32,
                data_fixed: fixed_cdf,
                data_random: random_cdf,
                image_path: plot_image_path.into(),
            }.plot().unwrap();
        println!(" ✓");
    println!("Finished benchmark for aes_add_round_key.");
}
