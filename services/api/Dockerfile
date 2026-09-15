FROM node:22-bookworm-slim
WORKDIR /app
RUN corepack enable && corepack prepare pnpm@9.15.9 --activate
COPY . .
RUN pnpm install --frozen-lockfile=false
ENV HOST=0.0.0.0
ENV PORT=4000
EXPOSE 4000
CMD ["pnpm", "--filter", "@markov/api", "start"]
