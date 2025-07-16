import { defineConfig } from 'vite'
import tsConfigPaths from 'vite-tsconfig-paths'

export default defineConfig({
  base: '/admin/threads/',
  server: {
    port: 3001,
  },
  build: {
    outDir: 'dist',
  },
  plugins: [tsConfigPaths()],
})
