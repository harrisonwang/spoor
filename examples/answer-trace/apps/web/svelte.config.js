import adapter from '@sveltejs/adapter-static'
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'

/** @type {import('@sveltejs/kit').Config} */
export default {
  preprocess: vitePreprocess(),
  kit: {
    // 纯客户端 SPA(ssr=false)：打成静态站，fallback 作单页入口，直接传 Cloudflare Pages。
    adapter: adapter({ fallback: 'index.html' }),
  },
}
