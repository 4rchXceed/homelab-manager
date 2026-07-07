#!/bin/sh
mkdir -p data
echo "DATA_BACKUP" > data/data.txt
cp sampledatas.pdf data/sampledatas.pdf
mkdir -p data/datas/subdir
echo "DATA_BACKUP3" > data/datas/subdir/backup.txt
sleep 3600
