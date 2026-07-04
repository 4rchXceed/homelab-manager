FROM alpine:3.24

WORKDIR /app

RUN apk add --no-cache curl

COPY run.sh .

RUN chmod +x run.sh

CMD ["./run.sh"]
