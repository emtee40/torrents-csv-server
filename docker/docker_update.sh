#!/bin/sh
sudo docker build ../ --file Dockerfile -t torrents-csv-server:latest
sudo docker-compose up -d
