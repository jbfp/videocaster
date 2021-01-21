import svelte from 'rollup-plugin-svelte';
import commonjs from '@rollup/plugin-commonjs';
import copy from 'rollup-plugin-copy'
import resolve from '@rollup/plugin-node-resolve';
import { terser } from 'rollup-plugin-terser';
import css from 'rollup-plugin-css-only';
import autoPreprocess from 'svelte-preprocess';
import replace from '@rollup/plugin-replace';
import typescript from '@rollup/plugin-typescript';
import os from "os";

const production = !process.env.ROLLUP_WATCH;
const buildDir = 'public/build';

export default {
    input: 'src/index.ts',
    output: {
        sourcemap: !production,
        format: 'iife',
        name: 'app',
        file: `${buildDir}/index.js`
    },
    plugins: [
        replace({
            // \\ on Windows, / on sane systems
            __sep: os.platform() === "win32" ? '\\\\' : '/',
        }),

        svelte({
            compilerOptions: {
                // enable run-time checks when not in production
                dev: !production
            },
            preprocess: autoPreprocess(),
        }),

        typescript({ sourceMap: !production }),

        copy({
            targets: [
                { src: "src/fonts/*", dest: buildDir },
            ],
        }),

        // we'll extract any component CSS out into
        // a separate file - better for performance
        css({ output: 'bundle.css' }),

        // If you have external dependencies installed from
        // npm, you'll most likely need these plugins. In
        // some cases you'll need additional configuration -
        // consult the documentation for details:
        // https://github.com/rollup/plugins/tree/master/packages/commonjs
        resolve({
            browser: true,
            dedupe: ['svelte']
        }),

        commonjs(),

        // If we're building for production (npm run build
        // instead of npm run dev), minify
        production && terser()
    ],
    watch: {
        clearScreen: false
    }
};
