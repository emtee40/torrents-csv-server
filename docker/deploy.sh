#!/bin/sh

# Rebuilding docker
sudo docker build ../ --file Dockerfile -t torrents-csv-server:latest -t dessalines/torrents-csv-server:latest
sudo docker push dessalines/torrents-csv-server:latest

# SSH and pull it
ssh tyler@95.183.53.52 "cd ~/torrents-csv-server && docker compose pull && docker compose up -d"

curl -d "Torrents-csv-server deploy completed." ntfy.sh/dessalines
