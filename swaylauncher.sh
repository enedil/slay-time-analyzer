#!/bin/bash
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

cd "$SCRIPT_DIR"

pkill --full "telegramchatstats-sampler sample"
RUST_LOG=trace target/release/telegramchatstats-sampler sample
