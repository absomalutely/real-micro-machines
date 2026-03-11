FROM node:22-slim

RUN corepack enable && corepack prepare pnpm@latest --activate

WORKDIR /app

COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
COPY client/package.json client/package.json

RUN pnpm install --frozen-lockfile

COPY client/ client/

EXPOSE 5173

CMD ["pnpm", "dev", "--host"]
