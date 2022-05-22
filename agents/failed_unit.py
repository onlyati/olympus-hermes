#!/usr/bin/python3

import subprocess
import os
import socket

#
# Check that memory category exist, if does not create
#
groups = os.popen("/usr/local/bin/hermes-cli list-groups").read()
groups_list = groups.split("\n")

if "system" not in groups_list:
    rc = subprocess.call(["/usr/local/bin/hermes-cli", "create-group", "system"], stdout=subprocess.DEVNULL)
    if rc != 0:
        print("Error during system group creation, exit 1")
        exit(1)

output = os.popen("/usr/bin/systemctl --state=failed").read()
output_list = output.split('\n')

for line in output_list:
    line_content = list(filter(None, line.split(" ")))
    if len(line_content) == 0:
        continue

    if (line_content[1] == "loaded") & (line_content[2] == "units"):
        subprocess.call(["/usr/local/bin/hermes-cli", "set-item", "system/" + socket.gethostname() + ".failed-service-count", line_content[0]])