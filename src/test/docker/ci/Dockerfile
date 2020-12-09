 ARG IMAGESRC

FROM $IMAGESRC

# Allow flux-security version, username, UID, and GID to be overidden on
#  docker build command line:
#
ARG USER=fluxuser
ARG UID=1000
ARG GID=1000
ARG BASE_IMAGE

USER root

# Add configured user to image with sudo access:
#
RUN if [ "$USER" != "fluxuser" ]; then set -x \
 && groupadd -g $GID $USER;\
 useradd -g $GID -u $UID -d /home/$USER -m $USER \
 && printf "$USER ALL= NOPASSWD: ALL\\n" >> /etc/sudoers; fi

# Make sure user in appropriate group for sudo on different platforms
RUN if [ "$USER" != "fluxuser" ]; then\
    case $BASE_IMAGE in \
     bionic*) adduser $USER sudo ;; \
     focal*)  adduser $USER sudo ;; \
     centos*) usermod -G wheel $USER ;; \
     *) (>&2 echo "Unknown BASE_IMAGE") ;; \
    esac; fi

# Install extra dependencies if necessary here.
#
# Do not forget to run `apt update` on Ubuntu/bionic.
# Do NOT run `yum upgrade` on CentOS (this will unnecessarily upgrade
#  existing packages)
#
RUN case $BASE_IMAGE in \
     bionic*) ;; \
     focal*) ;; \
     centos*) ;; \
     *) (>&2 echo "Unknown BASE_IMAGE") ;; \
    esac

ENV BASE_IMAGE=$BASE_IMAGE
USER $USER
WORKDIR /home/$USER

RUN flux keygen
