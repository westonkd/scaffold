FROM node:22.13.1-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends zip \
 && rm -rf /var/lib/apt/lists/*

COPY package.json ./
RUN npm install --ignore-scripts

COPY tsconfig.json vite.config.ts ./
COPY src ./src

RUN npm run build
RUN cd dist/build && zip -9 ../authorizer.zip index.js

FROM scratch AS export
COPY --from=builder /app/dist/authorizer.zip /authorizer.zip
