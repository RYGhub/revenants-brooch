FROM --platform=${BUILDPLATFORM} rust:bullseye AS builder
ARG BUILDPLATFORM
ARG TARGETPLATFORM

RUN apt-get update && \
    apt-get upgrade --assume-yes

RUN \
    mkdir .cargo && \
    echo '[net]' >> .cargo/config.toml && \
    echo 'git-fetch-with-cli = true' >> .cargo/config.toml && \
    echo >> .cargo/config.toml && \
    if [ "${BUILDPLATFORM}" != "${TARGETPLATFORM}" ]; then \
        if [ "${TARGETPLATFORM}" = "linux/amd64" ]; then \
            apt-get install --assume-yes gcc-x86-64-linux-gnu; \
            echo '[target.x86_64-unknown-linux-gnu]' >> .cargo/config.toml; \
            echo 'linker = "x86-64-linux-gnu-gcc"' >> .cargo/config.toml; \
            echo >> .cargo/config.toml; \
        fi && \
        if [ "${TARGETPLATFORM}" = "linux/arm64" ]; then \
            apt-get install --assume-yes gcc-aarch64-linux-gnu; \
            echo '[target.aarch64-unknown-linux-gnu]' >> .cargo/config.toml; \
            echo 'linker = "aarch64-linux-gnu-gcc"' >> .cargo/config.toml; \
            echo >> .cargo/config.toml; \
        fi && \
        if [ "${TARGETPLATFORM}" = "linux/arm/v7" ]; then \
            apt-get install --assume-yes gcc-arm-linux-gnueabihf; \
            echo '[target.armv7-unknown-linux-gnueabihf]' >> .cargo/config.toml; \
            echo 'linker = "arm-linux-gnueabihf-gcc"' >> .cargo/config.toml; \
            echo >> .cargo/config.toml; \
        fi \
    fi

RUN \
    if [ "${TARGETPLATFORM}" = "linux/amd64" ]; then \
        RUSTTARGET=x86_64-unknown-linux-gnu; \
    fi && \
    if [ "${TARGETPLATFORM}" = "linux/arm64" ]; then \
        RUSTTARGET=aarch64-unknown-linux-gnu; \
    fi && \
    if [ "${TARGETPLATFORM}" = "linux/arm/v7" ]; then \
        RUSTTARGET=armv7-unknown-linux-gnueabihf; \
    fi && \
    rustup target add ${RUSTTARGET}

WORKDIR /usr/src/revenants_brooch/
COPY ./ ./

RUN \
    if [ "${TARGETPLATFORM}" = "linux/amd64" ]; then \
        RUSTTARGET=x86_64-unknown-linux-gnu; \
    fi && \
    if [ "${TARGETPLATFORM}" = "linux/arm64" ]; then \
        RUSTTARGET=aarch64-unknown-linux-gnu; \
    fi && \
    if [ "${TARGETPLATFORM}" = "linux/arm/v7" ]; then \
        RUSTTARGET=armv7-unknown-linux-gnueabihf; \
    fi && \
    cargo build --all-features --bins --release --target=${RUSTTARGET}

#############################################################################

FROM --platform=${TARGETPLATFORM} rust:bullseye AS final

WORKDIR /usr/src/revenants_brooch/
COPY --from=builder \
    /usr/src/revenants_brooch/target/*/release/revenants_brooch \
    /usr/bin/

ENTRYPOINT ["revenants_brooch"]
CMD []

LABEL org.opencontainers.image.title="Revenant's Brooch"
LABEL org.opencontainers.image.description="Dota 2 guild match history webhook for Discord"
LABEL org.opencontainers.image.licenses="AGPL-3.0-or-later"
LABEL org.opencontainers.image.url="https://github.com/RYGhub/revenants-brooch"
LABEL org.opencontainers.image.authors="Stefano Pigozzi <me@steffo.eu>"
ENV RUST_LOG "warn,revenants_brooch=info"
