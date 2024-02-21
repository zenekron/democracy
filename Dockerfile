################################################################################
# builder image
################################################################################

# https://hub.docker.com/_/rust
FROM rust:1.76 as builder

WORKDIR /data

COPY ./ ./

RUN cargo install --path .



################################################################################
# runtime image
################################################################################

FROM debian:bullseye-slim

WORKDIR /data

COPY --from=builder /usr/local/cargo/bin/democracy /usr/local/bin/democracy

CMD ["democracy"]
