#!/usr/bin/env python

import time
import os
import pathlib

rundir = pathlib.Path(f"/run/user/{os.getuid()}/")

while True:
    socks = list(rundir.glob("sway-ipc*sock"))
    if len(socks) == 1:
        break 
    if len(socks) > 1:
        exit(f"more than one sway sock: {socks}")
    time.sleep(60)

env = os.environ
env["I3SOCK"] = str(socks[0])

os.execvpe("target/release/telegramchatstats", ["target/release/telegramchatstats"], env)
