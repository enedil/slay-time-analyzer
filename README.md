# Slay-time-analyzer
Slay time analyzer gives you insights into what windows (application, window title) you spend time on while using Sway or I3.

## Quick intro

Start with `rustup override set 1.71.0 && cargo build --release`.
There are two modules. Second one hardly works correctly, and lacks appropriate analyzer and will be perhaps described later. Version 1 is just a user process. Version 2 works as a systemd service (this is problematic, as it interferes with Selinux, and a few other concerns).

## Version 1:

Version 1 samples each 10ms the state of window tree, selects the focused window and records the info.
Add `exec $PATH_TO_CLONED_DIR/swaylauncher.sh` to your sway config. Analyze results with `cargo run --bin telegramchatstats-sampler analyze` from the repository directory, which shows usage stats based on last 24 hours of usage. Suggested usage is to use `watch cargo run --bin telegramchatstats-sampler analyze`. This version logs important messages to journalctl.

## Version 2:
Version 2 listens on events from Sway - when focus changes or window title changes, it records the change.
There's an attached script to generate and install SystemD service. Enable that service. No analyzer yet, but you can get raw results in `windowevents.db` sqlite database.
