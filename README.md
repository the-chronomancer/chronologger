# ChronoLogger

![Rust](https://img.shields.io/badge/rust-1.79%2B-blue.svg)
![clippy](https://img.shields.io/badge/clippy-passing-olivegreen.svg)
![rustfmt](https://img.shields.io/badge/rustfmt-passing-black.svg)
![tests](https://img.shields.io/badge/tests-passing-blueviolet.svg)
[![license: MIT](https://img.shields.io/crates/l/clippy.svg)](#license)

## Table of Contents
- [Overview](#overview)
- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Usage](#usage)
- [Command-Line Arguments](#command-line-arguments)
- [Examples](#examples)
  - [Default Parameters](#default-parameters)
  - [Custom Interval and Output File](#custom-interval-and-output-file)
  - [Short Duration](#short-duration)
- [License](#license)
- [Contact](#contact)

## Overview

ChronoLogger is a Rust command-line tool designed to log CPU and memory usage of processes to a CSV file at regular intervals.

## Features

- **CSV Writing**: Writes process information including timestamp, PID, process name, CPU usage, and memory usage.
- **Configurable Interval**: Set the logging interval in seconds.
- **Configurable Duration**: Set the maximum duration to run the logger.
- **Signal Handling**: Gracefully handles termination signals (SIGINT, SIGTERM).

## Requirements

- Rust 1.79+
- 
## Installation

1. Ensure Rust 1.56 or higher is installed on your system.
2. Install the crate from crates.io:
```bash
cargo install chronologger
```

## Usage

Run the application from the command line. Default values will be used if no arguments are provided.
```bash
chronologger
```
You can also specify the desired parameters. For example:
```bash
chronologger --interval 2 --output custom_output.csv --duration 120
```

## Command-Line Arguments

- `-i, --interval`: Sets the logging interval in seconds. Default: 1
- `-o, --output`: Sets the output CSV file. Default: 'process_usage.csv'
- `-d, --duration`: Sets the maximum duration to run in seconds. Default: 60

## Examples

### Default Parameters
Run ChronoLogger with default parameters. This will write process information every second for 60 seconds and save it to `process_usage.csv`.
```bash
chronologger
```

### Custom Interval and Output File
Write process information every 2 seconds for 120 seconds and save it to `custom_output.csv`.
```bash
chronologger --interval 2 --output custom_output.csv --duration 120
```

### Short Duration
Write process information for 10 seconds.
```bash
chronologger --duration 10
```

## License

This software is released under the MIT License.

## Contact

For support or contributions, please contact [Jacob Coleman](mailto:jacob.wade.coleman@gmail.com).
```