import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { resolve } from "path";
import { writeFileSync, readFileSync, mkdirSync } from "fs";
import type { Plugin } from "vite";

const host = process.env.TAURI_DEV_HOST;

const FEEDBACK_PATH = resolve(".claude/showcase-feedback.json");

function showcaseFeedbackPlugin(): Plugin {
  return {
    name: "showcase-feedback",
    configureServer(server) {
      server.middlewares.use("/api/showcase-feedback", (req, res) => {
        if (req.method === "POST") {
          let body = "";
          req.on("data", (chunk: string) => (body += chunk));
          req.on("end", () => {
            mkdirSync(resolve(".claude"), { recursive: true });
            writeFileSync(FEEDBACK_PATH, body);
            res.writeHead(200, { "Content-Type": "text/plain" });
            res.end("ok");
          });
        } else if (req.method === "GET") {
          try {
            const data = readFileSync(FEEDBACK_PATH, "utf-8");
            res.writeHead(200, { "Content-Type": "application/json" });
            res.end(data);
          } catch {
            res.writeHead(200, { "Content-Type": "application/json" });
            res.end("[]");
          }
        } else {
          res.writeHead(405);
          res.end();
        }
      });
    },
  };
}

export default defineConfig(async () => ({
  plugins: [tailwindcss(), svelte(), showcaseFeedbackPlugin()],
  resolve: {
    alias: {
      $lib: resolve("./src/lib"),
    },
  },
  build: {
    rollupOptions: {
      input: {
        main: resolve("./index.html"),
        showcase: resolve("./showcase.html"),
      },
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
