#!/bin/bash
#
#  Build flux-rs "ci" docker image and run tests, exporting
#   important environment variables to the docker environment.
#
#
# option Defaults:
IMAGE=focal
JOBS=2
MOUNT_CARGO_ARGS="--volume=$HOME/.cargo/:/home/$USER/.cargo"

#
declare -r prog=${0##*/}
die() { echo -e "$prog: $@"; exit 1; }

#
declare -r long_opts="help,quiet,interactive,image:,flux-security-version:,jobs:,no-cache,no-home,tag:"
declare -r short_opts="hqIdi:S:j:t:D:"
declare -r usage="
Usage: $prog [OPTIONS]\n\
Build docker image for ci builds, then run tests inside the new\n\
container as the current user and group.\n\
\n\
Uses the current git repo for the build.\n\
\n\
Options:\n\
 -h, --help                    Display this message\n\
     --no-cache                Disable docker caching\n\
     --no-cargo                Skip mounting the host cargo directory\n\
 -q, --quiet                   Add --quiet to docker-build\n\
 -i, --image=NAME              Use base docker image NAME (default=$IMAGE)\n\
 -j, --jobs=N                  Value for make -j (default=$JOBS)\n
 -I, --interactive             Instead of running ci build, run docker\n\
                                image with interactive shell.\n\
"

# check if running in OSX
if [[ "$(uname)" == "Darwin" ]]; then
    # BSD getopt
    GETOPTS=`/usr/bin/getopt $short_opts -- $*`
else
    # GNU getopt
    GETOPTS=`/usr/bin/getopt -u -o $short_opts -l $long_opts -n $prog -- $@`
    if [[ $? != 0 ]]; then
        die "$usage"
    fi
    eval set -- "$GETOPTS"
fi
while true; do
    case "$1" in
      -h|--help)                   echo -ne "$usage";          exit 0  ;;
      -q|--quiet)                  QUIET="--quiet";            shift   ;;
      -i|--image)                  IMAGE="$2";                 shift 2 ;;
      -j|--jobs)                   JOBS="$2";                  shift 2 ;;
      -I|--interactive)            INTERACTIVE="/bin/bash";    shift   ;;
      --no-cache)                  NO_CACHE="--no-cache";      shift   ;;
      --no-cargo)                  MOUNT_CARGO_ARGS="";        shift   ;;
      --)                          shift; break;                       ;;
      *)                           die "Invalid option '$1'\n$usage"   ;;
    esac
done

TOP=$(git rev-parse --show-toplevel 2>&1) \
    || die "not inside flux-rs git repository!"
which docker >/dev/null \
    || die "unable to find a docker binary"

echo "Building image $IMAGE for user $USER $(id -u)"
docker build \
    ${NO_CACHE} \
    ${QUIET} \
    --build-arg BASE_IMAGE=$IMAGE \
    --build-arg IMAGESRC="fluxrm/rust:$IMAGE" \
    --build-arg USER=$USER \
    --build-arg UID=$(id -u) \
    --build-arg GID=$(id -g) \
    -t flux-rs-ci:${IMAGE} \
    $TOP/src/test/docker/ci \
    || die "docker build failed"

if [[ -n "$MOUNT_CARGO_ARGS" ]]; then
    echo "Creating cargo directory"
    mkdir -p $HOME/.cargo
    echo "Mounting cargo directory with ${MOUNT_CARGO_ARGS}"
fi
echo "mounting $TOP as /usr/src"

export JOBS
export BUILD_DIR
export chain_lint

docker run --rm \
    --workdir=/usr/src \
    --volume=$TOP:/usr/src \
    $MOUNT_CARGO_ARGS \
    -e CC \
    -e CXX \
    -e LDFLAGS \
    -e CFLAGS \
    -e CPPFLAGS \
    -e COVERAGE \
    -e TEST_INSTALL \
    -e CPPCHECK \
    -e TRAVIS \
    -e USER \
    --cap-add SYS_PTRACE \
    --tty \
    ${INTERACTIVE:+--interactive} \
    --network=host \
    flux-rs-ci:${IMAGE} \
    ${INTERACTIVE:-./src/test/ci_run.sh} \
|| die "docker run failed"
