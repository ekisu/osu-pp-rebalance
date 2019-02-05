# osu-pp-rebalance

[![Build Status](https://travis-ci.com/ekisu/osu-pp-rebalance.svg?branch=master)](https://travis-ci.com/ekisu/osu-pp-rebalance)

Calculate profile/beatmap pp after the new rebalances.

## Setup

1. Be sure to clone all submodules recursively. Either clone with the `--recursive` flag, or run `git submodule update --init --recursive`.
2. Install [rustup](https://rustup.rs/). Use the *nightly* channel.
3. Install [.NET Core SDK 2.2](https://www.microsoft.com/net/learn/get-started).
4. Copy `src/config.sample.rs` to `src/config.rs`, and change the values accordingly.
5. Run with `cargo run`. PerformanceCalculator will be built automatically.
