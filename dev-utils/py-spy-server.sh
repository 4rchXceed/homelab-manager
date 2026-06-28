docker exec server-server-1 pkill python3
# docker exec -it server-server-1 top
docker exec -it server-server-1 /bin/sh -c ". /venv/bin/activate && cd src && py-spy record -o profile.svg -- python3 main.py"
