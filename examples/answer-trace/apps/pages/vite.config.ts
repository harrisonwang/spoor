import { sveltekit } from '@sveltejs/kit/vite'
import tailwindcss from '@tailwindcss/vite'
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  // 前后端同源：/api/* 由顶层 functions/ 提供。全栈本地开发用 `wrangler pages dev`
  // （同时跑静态 + functions）；`vite dev` 只跑前端，loadTrace 会回退到内置 fixture。
})
