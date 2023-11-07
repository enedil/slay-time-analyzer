Start with `cargo build --release`.
There are two modules. First one hardly works correctly, and lacks appropriate analyzer and will be described later.

# Version 1:

Add `exec $PATH_TO_CLONED_DIR/swaylauncher.sh` to your sway config. Analyze results with `cargo run --bin telegramchatstats-sampler analyze` from the repository directory.

# Version 2:
There's an attached script to generate and install SystemD service. Enable that service. No analyzer yet, but you can get raw results in `windowevents.db` sqlite database.
