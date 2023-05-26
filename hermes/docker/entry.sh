#!/usr/bin/bash
echo "Current directory: $(pwd)"

ls -l /usr/var/hermes/init.dat || mkdir -p "/usr/var/hermes" && echo "" > /usr/var/hermes/init.dat
ls -l /usr/var/hermes/hook.dat || mkdir -p "/usr/var/hermes" && echo "" > /usr/var/hermes/hook.dat

./hermes /etc/hermes/main.conf