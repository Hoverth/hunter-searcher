# from https://hub.docker.com/_/rust/
FROM rust
WORKDIR /usr/src/hunter-searcher
COPY . .
RUN cargo install --path .

CMD ["hunter-searcher"]
