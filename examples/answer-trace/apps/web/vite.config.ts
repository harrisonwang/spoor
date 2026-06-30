import { sveltekit } from '@sveltejs/kit/vite'
import tailwindcss from '@tailwindcss/vite'
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  server: {
    // 让原文 markdown 里的相对图片地址 /api/media?... 能从 :5173 打到 api(:8000)。
    proxy: {
      '/api': 'http://localhost:8000',
    },
  },
})
