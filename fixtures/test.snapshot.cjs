/* eslint-disable */

exports[
  'ES modules are converted to CommonJS only for targets without ES modules - should match snapshot - 0'
] = {
  '/main.mjs':
    '"use strict";\nObject.defineProperty(exports, "__esModule", {\n    value: true\n});\nObject.defineProperty(exports, "value", {\n    enumerable: true,\n    get: function() {\n        return value;\n    }\n});\nvar value = a !== null && a !== void 0 ? a : b;\n'
}

exports[
  'ES modules are converted to CommonJS only for targets without ES modules - should match snapshot - 1'
] = {
  '/main.mjs': 'export const value = a !== null && a !== void 0 ? a : b;\n'
}

exports['import.meta is rewritten to the CommonJS bindings of Bare - should match snapshot - 0'] = {
  '/main.mjs':
    '"use strict";\nObject.defineProperty(exports, "__esModule", {\n    value: true\n});\nObject.defineProperty(exports, "default", {\n    enumerable: true,\n    get: function() {\n        return _default;\n    }\n});\nvar _default = {\n    url: module.url.href,\n    main: require.main === module,\n    cache: require.cache,\n    dirname: __dirname,\n    filename: __filename.length,\n    resolve: require.resolve(\'./dep\'),\n    addon: require.addon(),\n    host: require.addon.host,\n    addonPath: require.addon.resolve(),\n    asset: require.asset(\'./file\'),\n    meta: {\n        url: module.url.href,\n        main: require.main === module,\n        cache: require.cache,\n        dirname: __dirname,\n        filename: __filename,\n        resolve: require.resolve,\n        addon: require.addon,\n        asset: require.asset\n    },\n    unknown: {\n        url: module.url.href,\n        main: require.main === module,\n        cache: require.cache,\n        dirname: __dirname,\n        filename: __filename,\n        resolve: require.resolve,\n        addon: require.addon,\n        asset: require.asset\n    }.unknown\n};\n'
}

exports['top-level await cannot be converted to CommonJS - should match snapshot - 0'] = [
  {
    url: '/main.mjs',
    start: 19,
    end: 43,
    message: 'Top-level await cannot be lowered to CommonJS'
  },
  {
    url: '/main.mjs',
    start: 48,
    end: 79,
    message: 'Top-level await cannot be lowered to CommonJS'
  }
]

exports['syntax errors report the module and position - should match snapshot - 0'] =
  'Unexpected token `=`. Expected yield, an identifier, [ or { at /broken.js:2:7'

exports['syntax errors report the module and position - should match snapshot - 1'] = [
  {
    url: '/broken.js',
    start: 18,
    end: 19,
    message: 'Unexpected token `=`. Expected yield, an identifier, [ or {'
  }
]

/* eslint-enable */
