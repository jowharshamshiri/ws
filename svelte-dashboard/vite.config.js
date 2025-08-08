import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

export default defineConfig({
  plugins: [svelte()],
  build: {
    outDir: '../src/static',
    emptyOutDir: false,
    rollupOptions: {
      input: 'src/main.js',
      output: {
        entryFileNames: 'ade-app.js',
        chunkFileNames: 'ade-[name].js',
        assetFileNames: 'ade-[name].[ext]'
      }
    }
  }
})