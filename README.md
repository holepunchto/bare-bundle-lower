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

Modules are produced in one of two formats, set with `format`. With `esm`, the default for every target but `es5`, ES modules stay ES modules. With `cjs`, the default and only option for `es5`, the bundle loads without ES module support, which suits engines that support modern syntax but no ES modules, such as `JavaScriptCore.framework`. For example, `{ target: 'esnext', format: 'cjs' }` converts module syntax and nothing else.

With `cjs`, ES modules are converted to CommonJS and `import()` is converted to `require()` in every module, including CommonJS modules. Lowered modules are recorded with type `script`, and the imports converted to `require()` resolve with the `require` condition in place of the `import` condition. Import attributes are passed on to `require()`. The `import.meta` properties of converted modules are rewritten to their CommonJS equivalents: `url` to `module.url.href`, `main` to `require.main === module`, `cache` to `require.cache`, `dirname` to `__dirname`, `filename` to `__filename`, `resolve()` to `require.resolve()`, `addon()` to `require.addon()`, and `asset()` to `require.asset()`. Top-level `await` cannot be converted and is reported as an error. A CommonJS module shares the resolutions of its own `require()` calls with its converted `import()` calls, so an `import()` of a specifier that the module resolves to a different module for `require()` cannot be converted either and is reported as an error.

## License

Apache-2.0
