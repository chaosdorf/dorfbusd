FROM rust:1.56.1 AS builder

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && touch src/main.rs && cargo fetch && rm -rf src

COPY src ./src
RUN cargo build --release
RUN strip target/release/dorfbusd

FROM debian:bullseye-slim

LABEL author="rappet <rappet@rappet.de>"

COPY --from=builder /target/release/dorfbusd /bin/dorfbusd

EXPOSE 8080
ENV HTTP_PORT=8080
ENV SERIAL_BOUD=9600
ENV SERIAL_PATH=/dev/ttyUSB0

CMD [ "/bin/dorfbusd" ]
