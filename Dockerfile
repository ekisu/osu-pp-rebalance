FROM liuchong/rustup:nightly AS build_calculator

# create empty project
RUN USER=root cargo new osu-pp-rebalance --bin
WORKDIR /osu-pp-rebalance

# copy our manifests, and a hello world to get cargo to build
COPY docker_assets/hello_world.rs src/main.rs
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# cache deps
RUN cargo build --release
RUN rm src/*.rs

# copy our sources
COPY ./src ./src
COPY ./build.rs ./build.rs

# build
RUN rm ./target/release/deps/osu_pp_rebalance*
RUN DONT_BUILD_PERFORMANCE_CALCULATOR=1 cargo build --release

FROM microsoft/dotnet:2.2-sdk AS build_dll
WORKDIR /osu-pp-rebalance
COPY osu-tools ./osu-tools
COPY --from=build_calculator /osu-pp-rebalance/target/release/osu-pp-rebalance /osu-pp-rebalance/binaries/

RUN dotnet publish osu-tools/PerformanceCalculator/PerformanceCalculator.csproj -c Release -o /osu-pp-rebalance/binaries

FROM microsoft/dotnet:2.2-runtime
COPY --from=build_dll /osu-pp-rebalance/binaries /osu-pp-rebalance

WORKDIR /osu-pp-rebalance
COPY ./templates ./templates
COPY ./static ./static
EXPOSE 8000
CMD [ "./osu-pp-rebalance" ]
