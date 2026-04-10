# SDK Verification Console

Deterministic browser console for `cow-sdk`.

Build:

```text
wasm-pack build --target web
```

Serve this directory over HTTP, for example:

```text
python -m http.server 8080
```

```text
bunx serve --listen 8080 .
```

Open [http://localhost:8080](http://localhost:8080).

Do not open `index.html` with `file://`.

When deployed through GitHub Pages, open:

```text
https://<owner>.github.io/<repo>/sdk-verification-console/
```
