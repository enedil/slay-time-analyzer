#!/bin/bash
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

cd "$SCRIPT_DIR"

pkill telegramchatstats-sampler
RUST_LOG=trace target/release/telegramchatstats-sampler sample
