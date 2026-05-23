# Client

Vue 3 + TypeScript + Vite frontend for the LineageOS Statistics dashboard.

## Development

Install [pnpm](https://pnpm.io/installation), then from this directory:

```sh
pnpm install
pnpm dev
```

The dev server runs at [http://localhost:5173](http://localhost:5173). Requests to `/api` are proxied to the backend at `http://localhost:8080` (see [vite.config.ts](vite.config.ts)), so make sure the backend is running via `cargo run` from the repo root.
