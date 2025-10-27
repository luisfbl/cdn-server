import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		proxy: {
			'/api/documents': {
				target: 'http://localstack:4566/restapis/3q5o86n2kp/prod/_user_request_/files',
				changeOrigin: true,
				rewrite: (path) => path.replace(/^\/api\/documents/, '')
			}
		}
	}
});
