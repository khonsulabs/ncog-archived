FROM ubuntu:18.04

RUN apt-get update \
    && apt-get install -y libpq-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY target/release/server .env ./
COPY webapp/static/ ./static

EXPOSE 7878/tcp

ENV RUST_BACKTRACE=1

CMD ./server