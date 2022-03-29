FROM rust:1.59 AS source
WORKDIR /usr/src/revenants-brooch
COPY . .

FROM source AS install
RUN cargo install --path .

FROM debian:buster AS app
RUN apt-get update
RUN apt-get install -y libssl1.1 ca-certificates
RUN rm -rf /var/lib/apt/lists/*
COPY --from=install /usr/local/cargo/bin/revenants_brooch /usr/local/bin/revenants_brooch
CMD ["revenants_brooch"]
