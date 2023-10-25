FROM rust:1.73 as builder

RUN USER=root cargo new --bin discord_bot
WORKDIR ./discord_bot
COPY ./Cargo.toml ./Cargo.toml

## Install target platform (Cross-Compilation) --> Needed for Alpine
RUN rustup target add x86_64-unknown-linux-musl

# This is a dummy build to get the dependencies cached.
RUN cargo build --target x86_64-unknown-linux-musl --release

RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/discord_bot*
RUN cargo build --target x86_64-unknown-linux-musl --release


FROM alpine:3.16.0 AS runtime
ARG APP=/usr/src/app

EXPOSE 8000

COPY --from=builder /discord_bot/target/x86_64-unknown-linux-musl/release/discord_bot ${APP}/discord_bot

USER $APP_USER
WORKDIR ${APP}

CMD ["./discord_bot"]