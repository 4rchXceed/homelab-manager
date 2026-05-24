#!/bin/sh
/app/cron.sh
if [ ! -f /etc/letsencrypt/live/webdash.dev/fullchain.pem ]; then
    bash -c "sleep 5 && certbot certonly --agree-tos -d webdash.dev -m lyam.zambaz@pm.me" &
fi
# mv /etc/nginx/conf.d/webdash.nginx.conf /etc/nginx/nginx.conf

/usr/local/openresty/bin/openresty -g "daemon off;"

