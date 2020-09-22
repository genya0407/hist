use argopt::cmd;

#[cmd]
fn main(
    #[opt(short = "b", long = "bin")] bin: Option<f64>,
    #[opt(short = "l", long = "bar-length", default_value = "80")] bar_length: i64,
) -> Result<(), ()> {
    let input = {
        use std::io::Read;
        let mut s = String::from("");
        std::io::stdin()
            .read_to_string(&mut s)
            .expect("failed to read from stdin");
        s
    };
    let histogram = Histgram::build(input, bin)?;
    let output = present_histogram(histogram, bar_length as f64);
    println!("{}", output);
    Ok(())
}

fn present_histogram(histogram: Histgram, max_bar_length: f64) -> String {
    if histogram.bars.is_empty() {
        return String::from("");
    }

    let prec = num::clamp(
        (-histogram.bin.log(10.0)).ceil() as i64 + 1, // When bin is 0.1, this results in 2
        0,  // Print as integer (bin is greater than or equal to 1)
        16, // Max f64 precision
    ) as usize;

    let max_freq = histogram
        .bars
        .iter()
        .max_by_key(|bar| bar.frequency)
        .unwrap() // histogram.bars is not empty here
        .frequency as f64;

    let mut lines: Vec<String> = vec![];
    let max_representative_string_length = histogram
        .bars
        .iter()
        .map(|bar| {
            format!(
                "{representative:.prec$}",
                representative = bar.representative,
                prec = prec
            )
            .len()
        })
        .max()
        .unwrap(); // hisotram.bars is not empty here

    // Body
    for bar in histogram.bars {
        let bar_length = calculate_bar_length(bar.frequency as f64, max_freq, max_bar_length);
        lines.push(format!(
            "{representative:>fill$.prec$}|{bar_string}",
            representative = bar.representative,
            fill = max_representative_string_length,
            prec = prec,
            bar_string = std::iter::repeat("*")
                .take(bar_length as usize)
                .collect::<String>()
        ));
    }
    // Footer
    lines.push(format!(
        "{space:>fill$}+{minus:->fill_minus$}+ {max_freq} times",
        space = " ",
        fill = max_representative_string_length,
        minus = "-",
        fill_minus = calculate_bar_length(max_freq, max_freq, max_bar_length),
        max_freq = max_freq as i64,
    ));
    lines.push(format!(
        "{space:>fill$}+{minus:->fill_minus$}+ {max_freq} times",
        space = " ",
        fill = max_representative_string_length,
        minus = "-",
        fill_minus = calculate_bar_length(max_freq, max_freq, max_bar_length) / 2,
        max_freq = (max_freq / 2.0) as i64,
    ));

    lines.join("\n")
}

fn calculate_bar_length(frequency: f64, max_freq: f64, max_bar_length: f64) -> usize {
    ((frequency as f64 / max_freq) * max_bar_length) as usize
}

const DEFAULT_OUTPUT_LINES: i64 = 30;

impl Histgram {
    pub fn build(input: String, bin: Option<f64>) -> Result<Self, ()> {
        let mut parsed: Vec<f64> = vec![];
        for line in input.lines() {
            let value: f64 = line.parse().map_err(|_| ())?;
            if value.is_nan() {
                return Err(()); // FIXME
            }
            parsed.push(value);
        }

        if parsed.is_empty() {
            return Ok(Self {
                bin: f64::NAN,
                bars: vec![],
            });
        }

        // f64 does not implement Ord
        parsed.sort_by(|a, b| {
            a.partial_cmp(b)
                .ok_or_else(|| format!("Uncomparable. {} with {}", a, b))
                .unwrap()
        });

        // parsed is not empty here
        let min = *parsed.first().unwrap();
        let max = *parsed.last().unwrap();

        let bin = bin.unwrap_or((max - min) / DEFAULT_OUTPUT_LINES as f64);

        let mut bars = vec![];
        let mut current_range_min = min;
        let mut current_range_max = min + bin;
        let mut current_bar = Bar {
            representative: (current_range_max + current_range_min) / 2.0,
            frequency: 0,
        };
        // Count put current_bar.frequency until parsed value exceeds current_range_max.
        // If value exceeds current_range_max, push the current_bar to bars, and reset current_bar.
        for value in parsed.into_iter() {
            // `greater than` (not 'greater than or equal') in order to treat cases where parsed values are identical
            // This may result in incorrect histogram.
            if value > current_range_max {
                bars.push(current_bar.clone());

                current_range_min = current_range_max;
                current_range_max += bin;
                current_bar = Bar {
                    representative: (current_range_max + current_range_min) / 2.0,
                    frequency: 0,
                };
            }

            current_bar.frequency += 1;
        }
        bars.push(current_bar);

        Ok(Self {
            bin: bin,
            bars: bars,
        })
    }
}

struct Histgram {
    bin: f64,
    bars: Vec<Bar>,
}

#[derive(Clone)]
struct Bar {
    representative: f64,
    frequency: u64,
}
