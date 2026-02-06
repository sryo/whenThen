# When ♒︎

Torrent automation with playlets — small when→then rules for your downloads.

- When a torrent name contains "1080p", cast it to the living room TV.
- When a torrent name starts with "OST", move it to your music folder.
- When any torrent is added, grab subtitles in English and Spanish.
- When a torrent is larger than 20 GB, move it to an external drive.
- When a download stalls, send a notification.

## Development

Svelte 5 / TypeScript / Tailwind CSS frontend, Rust backend with Tauri 2. Uses librqbit for torrents, rust_cast for Chromecast, and Axum to serve media over the local network.

Requires [Node.js](https://nodejs.org/) (v18+), [Rust](https://www.rust-lang.org/tools/install) (stable), and the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/).

```bash
npm install
npm run tauri dev
npm run tauri build
```
