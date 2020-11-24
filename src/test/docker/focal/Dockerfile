FROM fluxrm/flux-core:focal

LABEL maintainer="Stephen Herbein <herbein1@llnl.gov>"

USER root

# avoid debconf from asking for input
ENV DEBIAN_FRONTEND noninteractive

# Update pkg caches, install rust package manager
RUN apt-get update \
 && apt-get -qq install -y --no-install-recommends \
        cargo \
 && rm -rf /var/lib/apt/lists/*

ENV LIBCLANG_PATH /usr/lib/x86_64-linux-gnu/libclang-10.so.1

USER fluxuser
