/// <reference types="vitest" />
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';
import environment from 'vite-plugin-environment';
import dotenv from 'dotenv';

dotenv.config();

const IS_DEV       = process.env.DFX_NETWORK !== "ic";
const REPLICA_PORT = process.env.DFX_REPLICA_PORT ?? "4943";

export default defineConfig({
  root: 'src/frontend',
  build: {
    outDir: '../../dist',
    emptyOutDir: true,
  },
  optimizeDeps: {
    esbuildOptions: {
      define: {
        global: 'globalThis',
      },
    },
  },
  server: {
    proxy: {
      '/api': {
				target: IS_DEV ? `http://localhost:${REPLICA_PORT}` : `https://ic0.app`,
				changeOrigin: true,
				secure: !IS_DEV,
      },
    },
  },
  plugins: [
    react(),
    // Maps all envars prefixed with 'CANISTER_' to process.env.*
    environment("all", { prefix: "CANISTER_" }),
    // Weirdly process not available to Webworker but import.meta.env will be.
    environment("all", { prefix: "CANISTER_", defineOn: 'import.meta.env' }),
    // Maps all envars prefixed with 'DFX_' to process.env.*
    environment("all", { prefix: "DFX_" }),
    // Weirdly process not available to Webworker but import.meta.env will be.
    environment("all", { prefix: "DFX_", defineOn: 'import.meta.env' }),
		// Maps all envars prefixed with 'VITE_' to process.env.*
		environment({ DFX_REPLICA_PORT: "4943" }),
		// Weirdly process not available to Webworker but import.meta.env will be.
		environment({ DFX_REPLICA_PORT: "4943" }, { defineOn: 'import.meta.env' }),
  ],
  test: {
    environment: 'jsdom',
    setupFiles: 'setupTests.ts',
    cache: { dir: '../node_modules/.vitest' },
  },
});