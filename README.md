# stats-rs
This command line utility allows you to compute simple statistics pipe in from standard input. 
This is designed to work with large files so statistics are computed in an online fashion, and as such might not be
exact. However this should be enough to get an idea of what your data looks like without needing to much CPU or RAM. 


## Example
Let's generate random 1 000 000 random integers with `jot` and compute our statistics on it: 

```shell
jot -r 1000000 | stats | column -s $'\t' -t
```

We should get: 
```
Mean:      50.518597
Variance:  834.72095
Median:    50.120243
q1:        25.425714
q3:        75.67728
Count:     1000000
Min:       1
Max:       100
```

by default it will output statistics in a human readable format, but you can also 
output results in JSON format and parse / process it with `jq` *e.g.*: 
```shell
$ jot -r 1000000 | stats --json | jq
```
Which should output: 
```json
{
  "count": 1000000,
  "max": 100,
  "mean": 50.51859664916992,
  "median": 50.120243072509766,
  "min": 1,
  "q1": 25.42571449279785,
  "q3": 75.67727661132812,
  "variance": 834.720947265625
}
```

## Usage
```
Compute statistics from a list of numbers by piping in stdin.

This tool is meant for use with large files, as such it computes these statistics in a
streaming fashion, the results might not be exact but they should be good enough on 
large datasets to get an idea of the data you have.

Usage: stats [OPTIONS]

Options:
  -u, --use-doubles
          Use f64 instead of f32, increasing precision but also memory usage

  -j, --json
          Print results as parsable json

  -n, --hide-running
          Hide running values for metrics

  -p, --polling <POLLING>
          Set polling interval for showing running values of statistics

          [default: 1000]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## See also
This is a rewrite of my other *(very simple)* [stats utility](https://github.com/lucblassel/stats) but using online statistics and rust instead of C++. 