FROM rust:1.73 as builder

RUN USER=root cargo new --bin discord_bot
WORKDIR ./discord_bot
COPY ./Cargo.toml ./Cargo.toml

## Install target platform (Cross-Compilation) --> Needed for Alpine
RUN rustup target add x86_64-unknown-linux-musl

RUN apt update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*

# This is a dummy build to get the dependencies cached.
RUN cargo build --target x86_64-unknown-linux-musl --release
RUN rm src/*.rs

COPY src ./src

RUN rm ./target/x86_64-unknown-linux-musl/release/deps/discord_bot*
RUN cargo build --target x86_64-unknown-linux-musl --release


FROM alpine:3.16.0 AS runtime
ARG APP=/usr/src/app

EXPOSE 8000

COPY --from=builder /discord_bot/target/x86_64-unknown-linux-musl/release/discord_bot ${APP}/discord_bot
COPY static ${APP}/discord_bot/static

USER $APP_USER
WORKDIR ${APP}

CMD ["./discord_bot"]