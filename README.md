# osu-pp-rebalance

[![Build Status](https://travis-ci.com/ekisu/osu-pp-rebalance.svg?branch=master)](https://travis-ci.com/ekisu/osu-pp-rebalance)

Calculate profile/beatmap pp after the new rebalances.

## Setup

1. Be sure to clone all submodules recursively. Either clone with the `--recursive` flag, or run `git submodule update --init --recursive`.
2. Install [rustup](https://rustup.rs/). Use the *nightly* channel.
3. Install [.NET Core SDK 2.2](https://www.microsoft.com/net/learn/get-started).
4. Set the env flags accordingly. See below for details.
5. Run with `cargo run`. PerformanceCalculator will be built automatically.

## Env flags

| Variable                      | Description                                                                                | Default value  |
|-------------------------------|--------------------------------------------------------------------------------------------|----------------|
| OSU_PP_CALC_API_KEY           | The [osu! api key](https://osu.ppy.sh/p/api). **Required**                                 | Not set        |
| OSU_PP_CALC_DOTNET_COMMAND    | Name of the *dotnet* executable                                                            | "dotnet"       |
| OSU_PP_CALC_NUM_THREADS       | The number of workers that are spawned for profile PP calculations                         | 2              |
| OSU_PP_CALC_LOAD_SAVE_RESULTS | If calculated profile results should be loaded/saved from/to a file on program start/close | false          |
| OSU_PP_CALC_RESULTS_FILE      | Where to load/save profile results                                                         | "results.data" |
| OSU_PP_CALC_BEATMAPS_CACHE    | Folder to save beatmap (.osu) files                                                        | cache          |

## Using Docker

Alternatively, you can run this service with Docker. Steps:
1. Install [Docker](https://docs.docker.com/install/), if you haven't already.
2. Start a swarm with `docker swarm init` in a shell.
3. Set up your [osu! api key](https://osu.ppy.sh/p/api) as a secret. Run in bash/PowerShell:

```
echo "YOUR_API_KEY_HERE" | docker secret create osu_pp_calc_api_key -
```

4. After creating the secret, create a service with:
```
docker service create --name osu-pp-rebalance \
       --secret osu_pp_calc_api_key \
       --publish published=8000,target=8000 \
       ekisu/osu-pp-rebalance:latest
```

This will download the latest image and run the service. The website will be available at http://localhost:8000. You can change the target port, if you wish.
