import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
    clearScreen: false,
    server: {
        port: 5173,
        strictPort: true,
        watch: {
            ignored: ["**/src-tauri/**"]
        }
    },
    envPrefix: ['VITE_', 'TAURI_'],
    build: {
        // Reduce memory usage during build
        chunkSizeWarningLimit: 1000,
        sourcemap: false,
        minify: 'esbuild', // Faster and less memory intensive than terser
    }
});
