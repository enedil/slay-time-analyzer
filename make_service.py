#!/usr/bin/env python3
import shutil
import pathlib

code_directory = pathlib.Path(__file__).parent

systemd_unit = """
[Unit]
Description=Rejestrator Telegrama

[Service]
User=enedil
Group=enedil
WorkingDirectory=%s
ExecStart=%s/startstats.py
Restart=always

[Install]
WantedBy = graphical.target
"""

service_name = "TelegramTracer.service"

with open(service_name, "w") as f:
    f.write(systemd_unit.format())

shutil.copy(service_name, "/etc/systemd/system")
