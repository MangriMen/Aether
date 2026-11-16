// Serves a fake updater manifest (`latest.json`) so the "update available" UI can be checked
// without a release. The signature and the url are fake: the update is found, but cannot be
// downloaded. Usage: `node scripts/update-fixture/serve.mjs [version] [port]`, then start the
// app with `--config scripts/update-fixture/tauri.conf.json` (see README.md).
import { createServer } from 'node:http';

const version = process.argv[2] ?? '99.0.0';
const port = Number(process.argv[3] ?? 8787);
const origin = `http://127.0.0.1:${port}`;

const platform = { signature: 'ZmFrZQ==', url: `${origin}/fake-update.bin` };
const manifest = {
  version,
  notes: 'Fixture update.\n\n- First change\n- Second change',
  pub_date: new Date().toISOString(),
  platforms: {
    'windows-x86_64': platform,
    'linux-x86_64': platform,
    'darwin-x86_64': platform,
    'darwin-aarch64': platform,
  },
};

createServer((req, res) => {
  if (req.url?.startsWith('/latest.json')) {
    res.writeHead(200, { 'content-type': 'application/json' });
    res.end(JSON.stringify(manifest));
  } else {
    res.writeHead(404).end();
  }
}).listen(port, '127.0.0.1', () => {
  console.log(`latest.json v${version} at ${origin}/latest.json`);
});
