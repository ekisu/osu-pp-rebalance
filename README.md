# osu-pp-rebalance

[![Build Status](https://travis-ci.com/ekisu/osu-pp-rebalance.svg?branch=master)](https://travis-ci.com/ekisu/osu-pp-rebalance)

Calculate profile/beatmap pp after the new rebalances.

## Setup

1. Install [rustup](https://rustup.rs/). Use the *nightly* channel.
2. Clone this [osu-tools](https://github.com/ekisu/osu-tools) fork. It won't work with the original one. Build it as normal (go to the PerformanceCalculator directory and run `dotnet build`)
3. Copy `src/config.sample.rs` to `src/config.rs`, and change the values accordingly.
4. Run with `cargo run`.
