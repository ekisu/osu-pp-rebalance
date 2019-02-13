FROM liuchong/rustup:nightly AS build_calculator_deps

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

FROM microsoft/dotnet:2.2-sdk AS build_dll
WORKDIR /osu-pp-rebalance
COPY ./osu-tools/PerformanceCalculator/PerformanceCalculator.csproj ./osu-tools/PerformanceCalculator/PerformanceCalculator.csproj
COPY ./osu-tools/osu ./osu-tools/osu

# we need this for ILLink.Tasks
COPY ./osu-tools/PerformanceCalculator/nuget.config /root/.nuget/NuGet/NuGet.Config
# cache osu-tools deps
RUN dotnet restore ./osu-tools/PerformanceCalculator/PerformanceCalculator.csproj

# Workaround libunwind8 dotnet bug
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        libunwind8 \
    && rm -rf /var/lib/apt/lists/*

# actually copy the code (that might have changed...)
COPY ./osu-tools ./osu-tools
RUN dotnet publish osu-tools/PerformanceCalculator/PerformanceCalculator.csproj -c Release -r linux-x64 -f netcoreapp2.0 -o /osu-pp-rebalance/binaries
RUN dotnet run --project osu-tools/RemoveBuildFiles/RemoveBuildFiles.csproj /osu-pp-rebalance/binaries

FROM liuchong/rustup:nightly AS build_calculator
WORKDIR /osu-pp-rebalance
# copy cached deps from build_calculator_deps stage
COPY --from=build_calculator_deps / /

# copy our sources
COPY ./src ./src
COPY ./build.rs ./build.rs

# build
RUN rm ./target/release/deps/osu_pp_rebalance*
RUN DONT_BUILD_PERFORMANCE_CALCULATOR=1 cargo build --release

FROM microsoft/dotnet:2.2-runtime
COPY --from=build_dll /osu-pp-rebalance/binaries/* /osu-pp-rebalance/
COPY --from=build_calculator /osu-pp-rebalance/target/release/osu-pp-rebalance /osu-pp-rebalance/osu-pp-rebalance

# Workaround libunwind8 dotnet bug (again)
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        libunwind8 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /osu-pp-rebalance
COPY ./templates ./templates
COPY ./static ./static
EXPOSE 8000
CMD [ "./osu-pp-rebalance" ]
