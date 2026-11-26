# bare-bundle-lower

Lower the syntax of the modules of a Bare bundle to an older ECMAScript version, such as ES5. The lowering takes a bundle and returns a new bundle, so it fits between bundling and either writing a `.bundle` file or compiling the bundle with `bare-bundle-compile`. It is meant to run at build time.

```
npm i bare-bundle-lower
```

## Usage

```js
const Bundle = require('bare-bundle')
const lower = require('bare-bundle-lower')

const bundle = new Bundle().write('/main.mjs', 'export const greet = (name) => `hello ${name}`', {
  main: true
})

lower(bundle, { target: 'es5' }).read('/main.mjs').toString()
```

Only syntax is lowered. Runtime APIs, such as `Promise`, `Symbol`, and `WeakMap`, are not polyfilled and must be provided by the target engine. The helpers that lowered syntax relies on are inlined into each module that uses them.

ES modules are converted to CommonJS when lowering to a target without ES modules, which is any target before ES2015. Converted modules are recorded with type `script` and resolve their imports with the `require` condition in place of the `import` condition. Their `import.meta` properties are rewritten to their CommonJS equivalents: `url` to `module.url.href`, `main` to `require.main === module`, `cache` to `require.cache`, `dirname` to `__dirname`, `filename` to `__filename`, `resolve()` to `require.resolve()`, `addon()` to `require.addon()`, and `asset()` to `require.asset()`. Top-level `await` cannot be converted and is reported as an error.

## License

Apache-2.0
