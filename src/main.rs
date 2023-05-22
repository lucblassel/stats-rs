use std::fmt::{Debug, Display};
use std::ops::{AddAssign, SubAssign};
use std::str::FromStr;
use std::{io, io::prelude::*};

use anyhow::{bail, Result};
use clap::Parser;
use crossterm::{cursor, terminal, ExecutableCommand};
use num_traits::{Float, FromPrimitive};
use serde::Serialize;
use serde_json::{json, Value};
use watermill::mean::Mean;
use watermill::quantile::Quantile;
use watermill::stats::Univariate;
use watermill::variance::Variance;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FloatError {
    #[error("Could not parse number on line {lineno}: '{number}'")]
    ParsingError { lineno: usize, number: String },
}

#[derive(Parser)]
#[command(author, version, about)]
/// Compute statistics from a list of numbers by piping in stdin.  
///
/// This tool is meant for use with large files, as such it computes
/// these statistics in a streaming fashion, the results might not be
/// exact but they should be good enough on large datasets to get
/// an idea of the data you have.
struct Cli {
    /// Use f64 instead of f32, increasing precision but also memory usage
    #[arg(short, long)]
    use_doubles: bool,
    /// Print results as parsable json
    #[arg(short, long)]
    json: bool,
    /// Hide running values for metrics.
    #[arg(short = 'n', long)]
    hide_running: bool,
    /// Set polling interval for showing running values of statistics
    #[arg(short, long, default_value_t = 1000)]
    polling: usize,
    /// Skip first line, e.g. header of a csv file
    #[arg(short, long)]
    skip_header: bool,
}

struct Stats<T>
where
    T: Float + FromPrimitive + AddAssign + SubAssign,
{
    mean: Mean<T>,
    median: Quantile<T>,
    q1: Quantile<T>,
    q3: Quantile<T>,
    variance: Variance<T>,
    count: usize,
    min: T,
    max: T,
    initialized: bool,
}

impl<T> Stats<T>
where
    T: Float + FromPrimitive + AddAssign + SubAssign + Serialize + Display,
{
    pub fn default() -> Self {
        Self {
            mean: Mean::new(),
            median: Quantile::new(T::from_f32(0.5).unwrap()).unwrap(),
            q1: Quantile::new(T::from_f32(0.25).unwrap()).unwrap(),
            q3: Quantile::new(T::from_f32(0.75).unwrap()).unwrap(),
            variance: Variance::default(),
            count: 0,
            min: Float::infinity(),
            max: Float::neg_infinity(),
            initialized: false,
        }
    }

    pub fn update(&mut self, val: T) {
        self.mean.update(val);
        self.median.update(val);
        self.q1.update(val);
        self.q3.update(val);
        self.variance.update(val);
        self.count += 1;
        self.min = self.min.min(val);
        self.max = self.max.max(val);
        self.initialized = true;
    }

    pub fn to_json(&self) -> Value {
        json!({
            "mean": self.mean.get(),
            "variance": self.variance.get(),
            "median": self.median.get(),
            "q1": self.q1.get(),
            "q3": self.q3.get(),
            "count": self.count,
            "min": self.min,
            "max": self.max,
        })
    }

    fn stub(&self) -> String {
        let mut s = "".to_owned();

        s += "Mean:\tNA\n";
        s += "Variance:\tNA\n";
        s += "Median:\tNA\n";
        s += "q1:\tNA\n";
        s += "q3:\tNA\n";
        s += &format!("Count:\t{}\n", self.count);
        s += &format!("Min:\t{}\n", self.min);
        s += &format!("Max:\t{}", self.max);

        s
    }
}

impl<T> Display for Stats<T>
where
    T: Float + FromPrimitive + AddAssign + SubAssign + Display + Debug + Serialize + FromStr,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.initialized {
            let mut s = "".to_owned();

            s += &format!("Mean:\t{}\n", self.mean.get());
            s += &format!("Variance:\t{}\n", self.variance.get());
            s += &format!("Median:\t{}\n", self.median.get());
            s += &format!("q1:\t{}\n", self.q1.get());
            s += &format!("q3:\t{}\n", self.q3.get());
            s += &format!("Count:\t{}\n", self.count);
            s += &format!("Min:\t{}\n", self.min);
            s += &format!("Max:\t{}", self.max);

            writeln!(f, "{}", s)
        } else {
            writeln!(f, "{}", self.stub())
        }
    }
}

pub fn compute_stats<T>(json: bool, running: bool, polling: usize, skip_header: bool) -> Result<()>
where
    T: Float + FromPrimitive + AddAssign + SubAssign + Display + Debug + Serialize + FromStr,
{
    let mut stderr = io::stderr();
    let mut stats = Stats::default();

    let running_print_height = 9;

    if running {
        writeln!(stderr, "{}", stats)?;
    }

    let mut lines = io::stdin().lock().lines();
    if skip_header {
        lines.next();
    }

    for (lineno, line) in lines.enumerate() {
        let line = line?;

        let num = match line.parse::<T>() {
            Ok(v) => v,
            Err(_) => {
                bail!("Could not parse number on line {lineno}: '{line}'");
            }
        };

        if running && lineno % polling == 0 {
            stderr.execute(cursor::MoveUp(running_print_height))?;
            stderr.execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;
            writeln!(stderr, "{}", stats)?;
        }

        stats.update(num)
    }

    // Clear stderr
    if running {
        stderr.execute(cursor::MoveUp(running_print_height))?;
        stderr.execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;
    }

    if json {
        println!("{}", stats.to_json())
    } else {
        println!("{}", stats)
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.use_doubles {
        compute_stats::<f64>(cli.json, !cli.hide_running, cli.polling, cli.skip_header)?;
    } else {
        compute_stats::<f32>(cli.json, !cli.hide_running, cli.polling, cli.skip_header)?;
    };

    Ok(())
}
