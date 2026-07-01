import adapter from '@sveltejs/adapter-static'
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'

/** @type {import('@sveltejs/kit').Config} */
export default {
  preprocess: vitePreprocess(),
  kit: {
    // 纯客户端 SPA(ssr=false)：打成静态站；顶层 functions/ 作 Pages Functions 后端，同源部署。
    adapter: adapter({ fallback: 'index.html' }),
  },
}
