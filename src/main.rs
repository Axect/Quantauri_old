use peroxide::fuga::*;

const T: usize = 5;

fn main() {
    let mut df = DataFrame::read_parquet("data/KB.parquet").unwrap();
    df.as_types(vec![Str, F64]);
    df.print();

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

    let mut dg = DataFrame::new(vec![]);
    dg.push("date", Series::new(date));
    dg.push("close", Series::new(close));
    dg.push("mean", Series::new(mean));
    dg.push("std_dev", Series::new(std_dev));
    dg.push("dev", Series::new(dev));
    dg.push("alpha", Series::new(alpha));
    dg.push("calpha", Series::new(calpha));

    dg.print();

    dg.write_parquet("data/KB_alpha.parquet", CompressionOptions::Uncompressed).unwrap();
}

fn ts_mean(v: &Vec<f64>, interval: usize) -> Vec<f64> {
    let mut result = vec![0f64; v.len()];
    for i in 0 .. v.len() {
        if i < interval {
            result[i] = v[i];
        } else {
            result[i] = v[i-interval .. i].iter().sum::<f64>() / interval as f64;
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
        result[i] = numerator / denominator;
    }
    result
}
