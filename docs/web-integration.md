# Web Integration & Self-Hosting Guide

This guide explains how to deploy **Classic123** (`rusted-classic-123`) on a personal static website or via Docker, load `.ods` / `.csv` files directly from your server, configure required HTTP MIME types, and optionally persist edits back to your server.

---

## 1. Static Bundle Artifacts (`dist/`)

Building the web application (`trunk build --release`) produces a self-contained static directory with zero runtime backend requirements:

```text
dist/
├─ index.html                     # Minimal host page (<canvas> + single ARIA live region)
├─ classic123_wasm.js             # Thin wasm-bindgen loader & input forwarder
├─ classic123_wasm_bg.wasm        # Optimized Rust engine + TUI + ODF 1.3 codec
├─ style.css                      # Minimal fullscreen canvas reset & focus ring
└─ fonts/                         # Optional static .woff2 monospace font (never inside .wasm)
```

Copy `dist/` to any static web host (Nginx, Caddy, Apache, Cloudflare Pages, GitHub Pages, S3/R2, or our multi-stage Docker container).

---

## 2. Loading Files from Your Server

### Option A — Query String Parameter (Zero Configuration)

Point users directly to a spreadsheet hosted on your site (same origin or CORS-enabled URL):

```text
https://yoursite.example/sheets/?file=/data/budget-2026.ods
```

On startup, the thin web host:
1. Instantiates `classic123_wasm_bg.wasm` via `WebAssembly.instantiateStreaming`.
2. Executes `fetch(fileUrl)` -> `response.arrayBuffer()` -> `new Uint8Array(buffer)`.
3. Passes the bytes straight into `classic123_wasm::load_ods_bytes(bytes)` (or `load_csv_bytes` for `.csv`), recalculates the DAG, and renders frame 1 on the `<canvas>`.

### Option B — JavaScript Configuration Hook (`window.CLASSIC123_CONFIG`)

To embed Classic123 inside an existing page or configure server-side saving (`PUT`/`POST`), declare a lightweight configuration object before the module script loads:

```html
<script>
  window.CLASSIC123_CONFIG = {
    // Optional default file loaded on startup (overridden by ?file=... unless locked)
    initialFileUrl: "/data/budget-2026.ods",

    // Restrict remote ?file= URLs to same-origin paths for safety (recommended)
    allowCrossOriginFiles: false,

    // Optional default theme token ("classic-dark" | "classic-blue" | "amber" | "high-contrast-light")
    theme: "classic-dark",

    // Optional async hook invoked by /File/Push Server (file.pushRemote)
    onSaveOds: async (odsBytes, filename) => {
      const response = await fetch(`/api/sheets/${encodeURIComponent(filename)}`, {
        method: "PUT",
        headers: {
          "Content-Type": "application/vnd.oasis.opendocument.spreadsheet",
        },
        credentials: "same-origin",
        body: odsBytes, // Uint8Array containing a 100% ODF 1.3 compliant .ods package
      });

      if (!response.ok) {
        throw new Error(`Server returned HTTP ${response.status}`);
      }
    },
  };
</script>
```

> **Local Download Always Available:** Even without `onSaveOds`, `/File/Save ODS` (`Ctrl+S`) serializes the `.ods` archive in WASM memory and triggers an instant browser download (`Blob` + `<a download>`).

---

## 3. Required Server Headers & MIME Types

To enable streaming WebAssembly compilation (instant startup) and clean `.ods` downloads/fetches, ensure your static server sends the correct `Content-Type` headers and compression:

| Extension | Required `Content-Type` | Notes |
|---|---|---|
| `.wasm` | `application/wasm` | Required by `WebAssembly.instantiateStreaming()` |
| `.ods` | `application/vnd.oasis.opendocument.spreadsheet` | Standard OASIS ODF 1.3 MIME type |
| `.woff2` | `font/woff2` | Optional static monospace font |

### Nginx Example

```nginx
server {
    listen 80;
    root /usr/share/nginx/html;
    index index.html;

    include /etc/nginx/mime.types;
    types {
        application/wasm wasm;
        application/vnd.oasis.opendocument.spreadsheet ods;
    }

    gzip on;
    gzip_types text/plain text/css application/javascript application/wasm;

    location ~* \.(wasm|js|css|woff2)$ {
        add_header Cache-Control "public, max-age=31536000, immutable";
    }

    location ~* \.ods$ {
        add_header Cache-Control "no-cache";
    }
}
```

### Caddy Example

```caddyfile
:80 {
    root * /srv/classic123
    encode zstd gzip
    file_server

    @wasm path *.wasm
    header @wasm Content-Type application/wasm

    @ods path *.ods
    header @ods Content-Type application/vnd.oasis.opendocument.spreadsheet
}
```

### Cloudflare Pages / Netlify (`_headers`)

```text
/*.wasm
  Content-Type: application/wasm
  Cache-Control: public, max-age=31536000, immutable

/*.ods
  Content-Type: application/vnd.oasis.opendocument.spreadsheet
  Cache-Control: no-cache
```

---

## 4. CORS & Security Recommendations

- **Same-Origin by Default:** Keep `.ods` files on the same origin as Classic123 whenever possible (`allowCrossOriginFiles: false`). This avoids CORS preflights and prevents your instance from being used to fetch arbitrary third-party URLs via crafted `?file=` links.
- **Cross-Origin `.ods` Buckets:** If your spreadsheets live on a separate static bucket (e.g., S3/R2), configure `Access-Control-Allow-Origin: https://yoursite.example` on the bucket and explicitly whitelist allowed origins in your host page.
- **Authentication on Save:** If implementing `onSaveOds` with `PUT`/`POST`, rely on HTTP-only session cookies (`credentials: "same-origin"`) or short-lived tokens obtained at runtime—never hardcode API keys or passwords inside static files.
- **Content Security Policy (CSP):** Classic123 requires `'wasm-unsafe-eval'` in `script-src` only when required by older browsers for WebAssembly compilation, and never uses `eval()` or inline cell HTML.

---

## 5. Docker Self-Hosting

Run the preconfigured multi-stage container (verified automatically by `cargo xtask docker-check`):

```sh
docker build -t classic123:local .

# Serve Classic123 alongside your own directory of .ods spreadsheets
docker run -d --name classic123 \
  -p 8080:80 \
  -v /path/to/my/spreadsheets:/usr/share/nginx/html/sheets:ro \
  classic123:local
```

Then open any sheet in your browser:

```text
http://localhost:8080/?file=/sheets/finances.ods
```