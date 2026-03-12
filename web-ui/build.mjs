import { build } from 'esbuild';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const require = createRequire(import.meta.url);

const aliases = new Map([
  ['buffer', require.resolve('buffer/')],
  ['stream', require.resolve('stream-browserify')],
  ['process', require.resolve('process/browser')],
  ['events', require.resolve('events/')],
  ['util', require.resolve('util/')]
]);

await build({
  entryPoints: [path.join(__dirname, 'src', 'wallet.tsx')],
  outfile: path.join(__dirname, '..', 'static', 'wallet.js'),
  bundle: true,
  format: 'esm',
  platform: 'browser',
  target: ['es2020'],
  jsx: 'automatic',
  minify: true,
  sourcemap: false,
  logLevel: 'info',
  inject: [path.join(__dirname, 'src', 'polyfills.ts')],
  define: {
    'process.env.NODE_ENV': '"production"',
    global: 'globalThis'
  },
  plugins: [
    {
      name: 'node-browser-aliases',
      setup(pluginBuild) {
        pluginBuild.onResolve({ filter: /^[a-z][a-z0-9_-]*$/i }, (args) => {
          const resolved = aliases.get(args.path);
          return resolved ? { path: resolved } : null;
        });
      }
    }
  ]
});
