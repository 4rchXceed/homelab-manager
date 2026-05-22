FROM crowdsecurity/openresty

WORKDIR /app

RUN apk add --no-cache curl certbot


# # Let's encrypt!!! CertBot
# RUN python3 --version && python3 -m venv /opt/certbot/
# RUN /opt/certbot/bin/pip install --upgrade pip
# RUN /opt/certbot/bin/pip install certbot certbot-nginx
# RUN ln -s /opt/certbot/bin/certbot /usr/local/bin/certbot

COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY .htpasswd /app/.htpasswd
COPY crowdsec-nginx-bouncer.conf /etc/crowdsec/bouncers/crowdsec-openresty-bouncer.conf
COPY cron.sh /app/cron.sh
COPY ./entrypoint.sh /app/entrypoint.sh

#RUN mkdir -p /usr/local/openresty/bin/ && ln /usr/lib/nginx/bin/openresty /usr/local/openresty/bin/openresty
RUN chmod +x /app/cron.sh
RUN chmod +x /app/entrypoint.sh 
RUN chmod +x /docker_start.sh

EXPOSE 80

ENTRYPOINT [ "/app/entrypoint.sh" ]
