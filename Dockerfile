FROM rust:1.59 AS files
WORKDIR /usr/src/revenants-brooch
COPY . .

FROM files AS build
RUN cargo install --path .

FROM debian:buster AS system
RUN apt-get update
RUN apt-get install -y libssl1.1 ca-certificates
RUN rm -rf /var/lib/apt/lists/*
COPY --from=install /usr/local/cargo/bin/revenants_brooch /usr/local/bin/revenants_brooch

FROM system AS entrypoint
ENTRYPOINT ["revenants_brooch"]
CMD []

FROM entrypoint AS final
