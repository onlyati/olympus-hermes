#!/usr/bin/python3

import subprocess
import os
import socket

#
# Check that memory category exist, if does not create
#
groups = os.popen("/usr/local/bin/hermes-cli list-groups").read()
groups_list = groups.split("\n")

if "memory" not in groups_list:
    rc = subprocess.call(["/usr/local/bin/hermes-cli", "create-group", "memory"], stdout=subprocess.DEVNULL)
    if rc != 0:
        print("Error during memory group creation, exit 1")
        exit(1)

#
# Read the memory usage and save 
#
meminfo = open("/proc/meminfo", "r")
lines = meminfo.readlines()
meminfo.close()

mem_total = 0
mem_free = 0
mem_buff = 0
mem_chache = 0
mem_used = 0
mem_slab = 0

for line in lines:
    line_content = list(filter(None, line.split(" ")))
    if len(line_content) == 0:
        continue

    if line_content[0] == "MemTotal:":
        mem_total = line_content[1]
    
    if line_content[0] == "MemFree:":
        mem_free = line_content[1]

    if line_content[0] == "Buffers:":
        mem_buff = line_content[1]

    if line_content[0] == "Cached:":
        mem_chache = line_content[1]

    if line_content[0] == "Slab:":
        mem_slab = line_content[1]

mem_buffch = int(mem_buff) + int(mem_chache) + int(mem_slab)
mem_used = int(mem_total) - int(mem_free) - int(mem_buffch)

print(mem_total, mem_used, mem_free, mem_buffch)

subprocess.call(["/usr/local/bin/hermes-cli", "set-item", "memory/" + socket.gethostname() + ".total", mem_total])
subprocess.call(["/usr/local/bin/hermes-cli", "set-item", "memory/" + socket.gethostname() + ".free", mem_free])
subprocess.call(["/usr/local/bin/hermes-cli", "set-item", "memory/" + socket.gethostname() + ".used", str(mem_used)])
subprocess.call(["/usr/local/bin/hermes-cli", "set-item", "memory/" + socket.gethostname() + ".cache", str(mem_buffch)])