FROM debian:bookworm-slim

ARG APT_DEBIAN_MIRROR=http://deb.debian.org/debian
ARG APT_SECURITY_MIRROR=http://deb.debian.org/debian-security

RUN sed -i "s|http://deb.debian.org/debian-security|${APT_SECURITY_MIRROR}|g; s|http://deb.debian.org/debian|${APT_DEBIAN_MIRROR}|g" /etc/apt/sources.list.d/debian.sources
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates libssl3 \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app
