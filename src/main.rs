use peroxide::fuga::*;
use dialoguer::{Input, theme::ColorfulTheme};
use std::process::{Command, Stdio};
use std::io::{prelude::*, BufReader};
use chrono::prelude::*;

const T: usize = 5;

fn main() {
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
    df.as_types(vec![Str, F64]);

    let date: Vec<String> = df["date"].to_vec();
    let close: Vec<f64> = df["close"].to_vec();

    let mean = ts_mean(&close, T);
    let std_dev = ts_std_dev(&close, T);
    let dev = close.sub_v(&mean);

    let alpha = zip_with(|x, y| - x / (y + 1e-6), &dev, &std_dev);

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
    dg.push("mean", Series::new(mean));
    dg.push("std_dev", Series::new(std_dev));
    dg.push("dev", Series::new(dev));
    dg.push("alpha", Series::new(alpha));
    dg.push("calpha", Series::new(calpha));
    dg.push("buy_sell", Series::new(buy_sell));

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
