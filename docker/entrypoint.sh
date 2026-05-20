#!/bin/sh

case "${JAM_FUZZ_SPEC}" in
    full) exec spacejam-full "$@" ;;
    *)    exec spacejam-tiny "$@" ;;
esac
