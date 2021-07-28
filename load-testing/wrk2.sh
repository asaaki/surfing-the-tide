#!/usr/bin/bash
wrk \
  -c 100 \
  -t 100 \
  -R 100 \
  -d 20m \
  -L \
  http://localhost:8080/
