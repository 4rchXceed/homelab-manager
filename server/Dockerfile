FROM python:3.12-alpine3.23

WORKDIR /app

RUN python3 -m venv /venv

RUN apk add --no-cache rclone build-base # py-spy

COPY requirements.txt requirements.txt

RUN source /venv/bin/activate && pip install -r requirements.txt

CMD ["tail", "-f", "/dev/null"]
