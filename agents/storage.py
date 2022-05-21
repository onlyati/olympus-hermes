#!/usr/bin/python3

import subprocess
import os
import socket

#
# Set mount points
#
if socket.gethostname() == "atihome":
    mount_names = ["root", "work", "data"]
    mount_points = ["/", "/mnt/work", "/mnt/data"]
elif socket.gethostname() == "pihome":
    mount_names = ["root"]
    mount_points = ["/",]
else:
    print("Running on invalid hostname:", socket.gethostname())
    exit(1)

#
# Check that storage category exist, if does not create
#
groups = os.popen(os.environ["PWD"] + "/hermes_cli.py list-groups").read()
groups_list = groups.split("\n")

if "storage" not in groups_list:
    rc = subprocess.call([os.environ["PWD"] + "/hermes_cli.py", "create-group", "storage"], stdout=subprocess.DEVNULL)
    if rc != 0:
        print("Error during storage group creation, exit 1")
        exit(1)

#
# Process output of 'df' then upload the needed datta
#
fsystems = os.popen("/usr/bin/df | /usr/bin/tail -n +2").read()
fsystems_lines = fsystems.split("\n")

for line in fsystems_lines:
    line_content = list(filter(None, line.split(" ")))
    if len(line_content) == 0:
        continue

    current_point = line_content[len(line_content) - 1]
    if current_point in mount_points:
        key = "storage/" + socket.gethostname() + "." + mount_names[mount_points.index(current_point)] + ".total"
        subprocess.call([os.environ["PWD"] + "/hermes_cli.py", "set-item", key, line_content[2]], stdout=subprocess.DEVNULL)

        key = "storage/" + socket.gethostname() + "." + mount_names[mount_points.index(current_point)] + ".used"
        subprocess.call([os.environ["PWD"] + "/hermes_cli.py", "set-item", key, line_content[3]], stdout=subprocess.DEVNULL)


