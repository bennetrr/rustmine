# RustMine Authentication Service

## Running

Before the first run, copy the `.env.template` file as `.env` and fill out the placeholders.
See [below on how to generate the JWT secret](#generating-the-jwt-secret).

### Pull image from registry

```bash
docker compose \
  --profile api \
  --env-file .env \
  up -d
```

### Build image locally

```bash
docker compose \
  --profile api-local \
  --env-file .env \
  up -d
```

### Only database (for local development)

```bash
docker compose \
  -f docker-compose.yml \
  -f docker-compose.dev.yml \
  --profile db \
  --env-file .env \
  up -d
```

## Generating the JWT Secret

```bash
openssl genpkey -algorithm ed25519 -out jwt_private.pem
openssl pkey -in jwt_private.pem -pubout -out jwt_public.pem
```

The contents of the `jwt_private.pem` and `jwt_public.pem` are the values of
the `JWT_PRIVATE_KEY` and the `JWT_PUBLIC_KEY` environment variables respectively.
