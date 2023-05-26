#!/usr/bin/bash
echo "Current directory: $(pwd)"

ls -l /usr/var/hermes/init.dat || mkdir -p "/usr/var/hermes" && echo "" > /usr/var/hermes/init.dat
ls -l /usr/var/hermes/hook.dat || mkdir -p "/usr/var/hermes" && echo "" > /usr/var/hermes/hook.dat
ls -l /etc/hermes/main.conf || mkdir -p "/etc/hermes" && cp "/etc/defaults/hermes.conf" "/etc/hermes/main.conf"

./hermes /etc/hermes/main.conf