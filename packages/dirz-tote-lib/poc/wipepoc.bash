#!/bin/bash

cat /etc/mtab | grep tote-poc | awk '{print $1}' | xargs sudo umount -f
rm -rf ~/tmp/tote-poc
