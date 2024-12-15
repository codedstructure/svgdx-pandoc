#!/usr/bin/env bash

FILTER_BIN="svgdx-pandoc"

if ! command -v "${FILTER_BIN}" > /dev/null ; then
    echo "${FILTER_BIN} not found; install with 'cargo install ${FILTER_BIN}'" >&2
    exit 2
fi

for FMT in html epub pdf; do
    pandoc --filter "${FILTER_BIN}" example.md -o example.${FMT}
done
