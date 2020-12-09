FROM fluxrm/flux-core:centos8

LABEL maintainer="Stephen Herbein <herbein1@llnl.gov>"

USER root

# Update pkg caches, install rust package manager
RUN yum -y update \
    && yum -y install rust-toolset clang-devel \
    && yum clean all

ENV LIBCLANG_PATH /usr/lib64/libclang.so

USER fluxuser
