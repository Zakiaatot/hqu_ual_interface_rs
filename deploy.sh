#!/bin/bash
docker build -t hqu_ual_interface_rs .
docker run -id --name hqu_ual_interface_rs -p 8085:8085 hqu_ual_interface_rs
# docker logs hqu_ual_interface_rs
