#!/usr/bin/python3

import sys
import requests as req
import http.client
import urllib3
urllib3.disable_warnings()

valid_actions = ["get-item", "set-item", "rem-item", "filter-item", "list-groups", "create-group", "rem-group"]

# http.client.HTTPConnection.debuglevel = 5

#
# Verify that we have at least enough input parameters
#
if len(sys.argv) < 2:
    print("Not enough input parameter")
    print("Format:", sys.argv[0], "<action> <key> <value>")
    print("")
    print("Valid actions:")
    print("\n".join(valid_actions))
    exit(1)

group = ""
key = ""
value = ""
action = sys.argv[1]

#
# Verify that call is valid
#
if action not in valid_actions:
    print("Invalid action detected:", action)
    print("Possible actions:", valid_actions)
    exit(1)

key_need = ["get-item", "set-item", "rem-item", "filter-item", "create-group", "rem-group"]
if action in key_need: 
    if len(sys.argv) < 3:
        print("Key must be specify for", action, "action but missing")
        exit(1)
    else:
        group = sys.argv[2]

slash_needed = ["get-item", "set-item", "rem-item", "filter-item"]
if action in slash_needed:
    if '/' not in group:
        print("Group/Key must be define but key is missing: <group>/<key>")
        exit(1)
    else:
        temp = group.split('/')
        group = temp[0]
        key = temp[1]


value_need = ["set-item"]
if action in value_need: 
    if len(sys.argv) < 4:
        print("Value must be specify for", action, "action but missing")
        exit(1)
    else:
        value = " ".join(sys.argv[3:])

#
# Get the request type and url
#

actions = ["get-item", 
           "set-item", 
           "rem-item", 
           "filter-item", 
           "list-groups", 
           "create-group", 
           "rem-group"]
http_types = ["get", 
              "post", 
              "delete", 
              "get", 
              "get", 
              "post", 
              "delete"]
http_urls = ["http://atihome.lan:9100/item?name=" + key + "&group=" + group,
             "http://atihome.lan:9100/item?name=" + key + "&group=" + group,
             "http://atihome.lan:9100/item?name=" + key + "&group=" + group,
             "http://atihome.lan:9100/filter?name=" + key + "&group=" + group,
             "http://atihome.lan:9100/group",
             "http://atihome.lan:9100/group?name=" + group,
             "http://atihome.lan:9100/group?name=" + group]

count = actions.index(action)
http_type = http_types[count]
http_url = http_urls[count]

resp = ""

if http_type == "get":
    resp = req.get(http_url)
elif http_type == "post":
    resp = req.post(http_url, data = value, stream=False)
elif http_type == "delete":
    resp = req.delete(http_url)
else:
    print("Error: invalid http type:", http_type)

print(resp.status_code)
print(resp.content.decode())