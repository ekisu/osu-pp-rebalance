FROM liuchong/rustup:nightly AS build_calculator

WORKDIR /app
COPY src ./src
COPY build.rs .
COPY Cargo.toml .

#RUN rustup default nightly
RUN DONT_BUILD_PERFORMANCE_CALCULATOR=1 cargo build --release

FROM microsoft/dotnet:2.2-sdk
COPY osu-tools ./osu-tools
COPY --from=build_calculator /app/target/release/osu-pp-rebalance ./binaries/

RUN dotnet build osu-tools/PerformanceCalculator/PerformanceCalculator.csproj -o ./binaries

CMD [ "./binaries/osu-pp-rebalance" ]
