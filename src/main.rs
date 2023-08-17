use peroxide::fuga::*;
use dialoguer::{Input, Select, theme::ColorfulTheme};
use std::process::{Command, Stdio};
use std::io::{prelude::*, BufReader};
use chrono::prelude::*;
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle, ParallelProgressIterator};

const T: usize = 5;

fn main() {
    let option = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select option")
        .default(0)
        .item("Add stock via code")
        .item("Update existing stocks")
        .interact()
        .unwrap();

    match option {
        0 => {
            let code = Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Input stock code")
                .default("005930".to_string())
                .interact()
                .unwrap();

            let start = Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Input start date")
                .default("2000-01-01".to_string())
                .interact()
                .unwrap();

            // Default: today
            let end = Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Input end date")
                .default(Local::now().format("%Y-%m-%d").to_string())
                .interact()
                .unwrap();

            // Download data using `python srcipt/observe.py --code {code}`
            let output = Command::new("python")
                .arg("script/observe.py")
                .arg("--code")
                .arg(&code)
                .arg("--start")
                .arg(&start)
                .arg("--end")
                .arg(&end)
                .stdout(Stdio::piped())
                .spawn()
                .unwrap()
                .stdout
                .unwrap();
            let reader = BufReader::new(output);
            for line in reader.lines() {
                println!("{}", line.unwrap());
            }

            // Read data
            let mut df = DataFrame::read_parquet(&format!("data/{}/close.parquet", code)).unwrap();
            df.as_types(vec![Str, F64, F64, F64]);

            let date: Vec<String> = df["date"].to_vec();
            let close: Vec<f64> = df["close"].to_vec();
            let high: Vec<f64> = df["high"].to_vec();
            let low: Vec<f64> = df["low"].to_vec();

            let dg = obtain_alpha(date, close, high, low, T);

            dg.print();

            dg.write_parquet(&format!("data/{}/alpha.parquet", code), CompressionOptions::Uncompressed).unwrap();

            // Plot using `python script/plot.py --code {code}`
            // Wait until plot is done
            let _ = Command::new("python")
                .arg("script/plot.py")
                .arg("--code")
                .arg(&code)
                .stdout(Stdio::piped())
                .spawn()
                .unwrap()
                .wait()
                .unwrap();

            println!("Done!");
        }
        1 => {
            // Read folders (data/{code})
            let mut folders = vec![];
            for entry in std::fs::read_dir("data").unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    folders.push(path);
                }
            }

            // Download data using `python srcipt/observe.py --code {code}`
            let pb = ProgressBar::new(folders.len() as u64);
            pb.set_style(ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"));
            for folder in folders.clone() {
                pb.set_message(format!("Downloading {}", folder.file_name().unwrap().to_str().unwrap()));
                let code = folder.file_name().unwrap().to_str().unwrap().to_string();
                let output = Command::new("python")
                    .arg("script/observe.py")
                    .arg("--code")
                    .arg(&code)
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap()
                    .stdout
                    .unwrap();
                let reader = BufReader::new(output);
                for line in reader.lines() {
                    println!("{}", line.unwrap());
                }
                pb.inc(1);
            }

            pb.finish_with_message("Download done!");

            // Read data
            let mut dfs = vec![];
            for folder in folders.clone() {
                let code = folder.file_name().unwrap().to_str().unwrap().to_string();
                let mut df = DataFrame::read_parquet(&format!("data/{}/close.parquet", code)).unwrap();
                df.as_types(vec![Str, F64, F64, F64]);
                dfs.push(df);
            }

            // Calculate alpha
            let pb = ProgressBar::new(folders.len() as u64);
            pb.set_style(ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} Computing alpha")
                .unwrap()
                .progress_chars("##-"));
            let dgs = dfs.into_par_iter()
                .progress_with(pb)
                .map(|df| {
                    let date: Vec<String> = df["date"].to_vec();
                    let close: Vec<f64> = df["close"].to_vec();
                    let high: Vec<f64> = df["high"].to_vec();
                    let low: Vec<f64> = df["low"].to_vec();
                    obtain_alpha(date, close, high, low, T)
                })
                .collect::<Vec<_>>();

            println!("Compute done!");

            // Write alpha
            let pb = ProgressBar::new(folders.len() as u64);
            pb.set_style(ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"));
            for (i, dg) in dgs.iter().enumerate() {
                pb.set_message(format!("Writing {}", folders[i].file_name().unwrap().to_str().unwrap()));
                let code = folders[i].file_name().unwrap().to_str().unwrap().to_string();
                dg.write_parquet(&format!("data/{}/alpha.parquet", code), CompressionOptions::Uncompressed).unwrap();
                pb.inc(1);
            }

            pb.finish_with_message("Write done!");

            // Plot using `python script/plot.py --code {code}`
            // Wait until plot is done
            let pb = ProgressBar::new(folders.len() as u64);
            pb.set_style(ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"));
            folders.par_iter().for_each(|folder| {
                pb.set_message(format!("Plotting {}", folder.file_name().unwrap().to_str().unwrap()));
                let code = folder.file_name().unwrap().to_str().unwrap().to_string();
                let _ = Command::new("python")
                    .arg("script/plot.py")
                    .arg("--code")
                    .arg(&code)
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
                pb.inc(1);
            });

            pb.finish_with_message("Plot done!");
        }
        _ => {}
    }
}

fn ts_mean(v: &Vec<f64>, interval: usize) -> Vec<f64> {
    let mut result = vec![0f64; v.len()];
    for i in 0 .. v.len() {
        if i < interval {
            result[i] = v[i];
        } else {
            result[i] = v[i-interval+1 ..=i].iter().sum::<f64>() / interval as f64;
        }
    }
    result
}

fn ts_std_dev(v: &Vec<f64>, interval: usize) -> Vec<f64> {
    let mut result = vec![0f64; v.len()];
    for i in 0 .. v.len() {
        if i < interval {
            result[i] = 0f64;
        } else {
            let mean = v[i-interval .. i].iter().sum::<f64>() / interval as f64;
            let mut sum = 0f64;
            for j in i-interval .. i {
                sum += (v[j] - mean).powi(2);
            }
            result[i] = (sum / interval as f64).sqrt();
        }
    }
    result
}

fn ts_diff(v: &Vec<f64>, interval: usize) -> Vec<f64> {
    let mut result = vec![0f64; v.len()];
    for i in 0 .. v.len() {
        if i < interval {
            result[i] = 0f64;
        } else {
            result[i] = v[i] - v[i-interval];
        }
    }
    result
}

fn ts_gaussian_smoothing(v: &Vec<f64>, interval: usize) -> Vec<f64> {
    let mut result = vec![0f64; v.len()];
    let kernel = |x: f64, y: f64| (- (x - y).powi(2) / (2f64 * (interval as f64).powi(2))).exp();
    for i in 0 .. v.len() {
        let mut numerator = 0f64;
        let mut denominator = 0f64;
        for j in 0 .. v.len() {
            if i > j + 3 * interval || j > i + 3 * interval {
                continue;
            }
            let weight = kernel(i as f64, j as f64);
            numerator += v[j] * weight;
            denominator += weight;
        }
        result[i] = numerator / (denominator + 1e-9);
    }
    result
}

#[allow(non_snake_case)]
fn obtain_alpha(date: Vec<String>, close: Vec<f64>, high: Vec<f64>, low: Vec<f64>, interval: usize) -> DataFrame {
    // For close
    let mean = ts_mean(&close, interval);
    let std_dev = ts_std_dev(&close, interval);
    let dev = close.sub_v(&mean);
    let alpha = zip_with(|x, y| - x / (y + 1e-6), &dev, &std_dev);
    let (calpha, calpha2) = alpha_to_calpha(&alpha);

    // Buy-Sell Signal
    // 1. Buy when calpha : + -> -
    // 2. Sell when calpha : - -> +
    let mut buy_sell = vec![0f64; calpha.len()];
    let mut is_buy = false;
    for i in 0 .. calpha.len() {
        if calpha[i] < 0f64 && !is_buy {
            buy_sell[i] = 1f64;
            is_buy = true;
        } else if calpha[i] >= 0f64 && is_buy {
            buy_sell[i] = -1f64;
            is_buy = false;
        }
    }
    let mut dg = DataFrame::new(vec![]);
    dg.push("date", Series::new(date));
    dg.push("close", Series::new(close));
    dg.push("high", Series::new(high));
    dg.push("low", Series::new(low));
    dg.push("mean", Series::new(mean));
    dg.push("std_dev", Series::new(std_dev));
    dg.push("alpha", Series::new(alpha));
    dg.push("calpha", Series::new(calpha));
    dg.push("calpha2", Series::new(calpha2));
    dg.push("buy_sell", Series::new(buy_sell));

    dg
}

fn alpha_to_calpha(alpha: &Vec<f64>) -> (Vec<f64>, Vec<f64>) {
    // Cumulative alpha
    let calpha = alpha.iter()
        .scan(0f64, |acc, &x| {
            *acc += x;
            Some(*acc)
        }).collect::<Vec<f64>>();
    let calpha_mean = ts_mean(&calpha, 20);
    let calpha_diff = calpha.sub_v(&calpha_mean);
    let calpha_grad = ts_diff(&calpha_diff, 1);
    let calpha = ts_gaussian_smoothing(&calpha_grad, 5);
    let calpha2 = ts_gaussian_smoothing(&calpha_grad, 10);
    (calpha, calpha2)
}
