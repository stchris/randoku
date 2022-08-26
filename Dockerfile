FROM rust:alpine as build
RUN apk add --no-cache musl-dev

RUN USER=root cargo new --bin randoku
WORKDIR /randoku

COPY Cargo.lock Cargo.toml .
RUN cargo build --release

COPY ./src ./src
COPY ./templates ./templates
RUN rm ./target/release/deps/randoku*
RUN cargo test && cargo build --release

FROM scratch

COPY --from=build /randoku/target/release/randoku /randoku
ENV ROCKET_PORT=8000
ENV ROCKET_ADDRESS=0.0.0.0

CMD ["/randoku"]
