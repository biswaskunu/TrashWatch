# TrashWatch — Deployment Guide

**Version:** 1.0

---

## 1. Local Development

```bash
# Build & run
cargo run

# Run with spectator auto-open
cargo run -- --spectator

# Custom port
cargo run -- --port 8080
```

---

## 2. Production Build

```bash
# Optimized release build
cargo build --release

# Binary at target/release/trashwatch
./target/release/trashwatch --port 3000
```

---

## 3. Cloudflare Workers Deployment

### 3.1 Prerequisites
- Cloudflare account
- `wrangler` CLI installed (`npm install -g wrangler`)
- Origin server running (Fly.io, Render, Railway, etc.) with public HTTPS/WSS

### 3.2 Configuration

**`wrangler.toml`:**
```toml
name = "trashwatch"
main = "deploy/worker.js"
compatibility_date = "2024-01-01"

[assets]
directory = "."  # spectator.html at root
binding = "ASSETS"

[vars]
ORIGIN = "wss://your-origin.fly.dev"  # Your Axum server WSS endpoint
```

### 3.3 Deploy
```bash
wrangler deploy
```

### 3.4 DNS
- Add CNAME: `trashwatch.yourdomain.com` → `<worker>.workers.dev`
- Enable "Always Use HTTPS"

---

## 4. Origin Server (Fly.io Example)

**`fly.toml`:**
```toml
app = "trashwatch"
primary_region = "iad"

[build]
  builder = "paketobuildpacks/builder:base"

[http_service]
  internal_port = 3000
  force_https = true
  auto_stop_machines = true
  auto_start_machines = true
  min_machines_running = 0
```

**Deploy:**
```bash
fly launch --no-deploy
fly deploy
```