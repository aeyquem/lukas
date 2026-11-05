FROM node:24.22.2-bookworm-slim AS development

WORKDIR /workspace

COPY app/package.json app/package-lock.json ./
RUN npm ci

COPY app/ ./

EXPOSE 5173

CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]

FROM development AS build

RUN npm run build

FROM nginx:1.31.8-alpine3.24 AS production

ENV NGINX_ENVSUBST_FILTER=^API_UPSTREAM$

RUN apk add --no-cache ca-certificates \
    && update-ca-certificates \
    && rm -f /etc/nginx/conf.d/default.conf

COPY --from=build /workspace/dist/ /usr/share/nginx/html/
COPY nginx/default.conf.template /etc/nginx/templates/default.conf.template
COPY nginx/api-proxy.conf /etc/nginx/snippets/api-proxy.conf
COPY docker-entrypoint.d/40-runtime-config.sh /docker-entrypoint.d/40-runtime-config.sh

RUN chmod 0755 /docker-entrypoint.d/40-runtime-config.sh

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -q --spider http://127.0.0.1:8080/ || exit 1
