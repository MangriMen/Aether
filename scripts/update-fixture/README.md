# Update fixture

Checks the "update available" UI (toast, settings, banner) without a release.

```sh
node scripts/update-fixture/serve.mjs [version] [port]   # default 99.0.0 on 8787
cd apps/desktop && bun run dev -- --config ../../scripts/update-fixture/tauri.conf.json
```

The override only changes the updater endpoint (merged over `tauri.conf.json`). The manifest's
signature is fake, so the update is found but installing it fails after the download.
A different port needs the same port in `tauri.conf.json` here.
