/* eslint-disable */

exports[
  'ES modules are converted to CommonJS only with the cjs format - should match snapshot - 0'
] = {
  '/main.mjs':
    '"use strict";\nObject.defineProperty(exports, "__esModule", {\n    value: true\n});\nObject.defineProperty(exports, "value", {\n    enumerable: true,\n    get: function() {\n        return value;\n    }\n});\nvar value = a !== null && a !== void 0 ? a : b;\n'
}

exports[
  'ES modules are converted to CommonJS only with the cjs format - should match snapshot - 1'
] = {
  '/main.mjs': 'export const value = a !== null && a !== void 0 ? a : b;\n'
}

exports[
  'ES modules are converted to CommonJS only with the cjs format - should match snapshot - 2'
] = {
  '/main.mjs':
    '"use strict";\nObject.defineProperty(exports, "__esModule", {\n    value: true\n});\nObject.defineProperty(exports, "value", {\n    enumerable: true,\n    get: function() {\n        return value;\n    }\n});\nconst value = a ?? b;\n'
}

exports[
  'import attributes are passed on to require() when converting to CommonJS - should match snapshot - 0'
] = {
  '/main.mjs':
    '"use strict";\nfunction _export_star(from, to) {\n    Object.keys(from).forEach(function(k) {\n        if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {\n            Object.defineProperty(to, k, {\n                enumerable: true,\n                get: function() {\n                    return from[k];\n                }\n            });\n        }\n    });\n    return from;\n}\nfunction _interop_require_default(obj) {\n    return obj && obj.__esModule ? obj : {\n        default: obj\n    };\n}\nfunction _getRequireWildcardCache(nodeInterop) {\n    if (typeof WeakMap !== "function") return null;\n    var cacheBabelInterop = new WeakMap();\n    var cacheNodeInterop = new WeakMap();\n    return (_getRequireWildcardCache = function(nodeInterop) {\n        return nodeInterop ? cacheNodeInterop : cacheBabelInterop;\n    })(nodeInterop);\n}\nfunction _interop_require_wildcard(obj, nodeInterop) {\n    if (!nodeInterop && obj && obj.__esModule) return obj;\n    if (obj === null || typeof obj !== "object" && typeof obj !== "function") return {\n        default: obj\n    };\n    var cache = _getRequireWildcardCache(nodeInterop);\n    if (cache && cache.has(obj)) return cache.get(obj);\n    var newObj = {\n        __proto__: null\n    };\n    var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor;\n    for(var key in obj){\n        if (key !== "default" && Object.prototype.hasOwnProperty.call(obj, key)) {\n            var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null;\n            if (desc && (desc.get || desc.set)) Object.defineProperty(newObj, key, desc);\n            else newObj[key] = obj[key];\n        }\n    }\n    newObj.default = obj;\n    if (cache) cache.set(obj, newObj);\n    return newObj;\n}\nObject.defineProperty(exports, "__esModule", {\n    value: true\n});\nfunction _export(target, all) {\n    for(var name in all)Object.defineProperty(target, name, {\n        enumerable: true,\n        get: Object.getOwnPropertyDescriptor(all, name).get\n    });\n}\n_export(exports, {\n    get bytes () {\n        return _filebin.default;\n    },\n    get lazy () {\n        return lazy;\n    },\n    get text () {\n        return _filetxt.default;\n    }\n});\nvar _filetxt = /*#__PURE__*/ _interop_require_default(require("./file.txt", {\n    with: {\n        type: \'text\'\n    }\n}));\n_export_star(require("./data.json", {\n    with: {\n        type: \'json\'\n    }\n}), exports);\nvar _filebin = /*#__PURE__*/ _interop_require_default(require("./file.bin", {\n    with: {\n        type: \'binary\'\n    }\n}));\nvar lazy = function lazy() {\n    return Promise.resolve().then(function() {\n        return _interop_require_wildcard(require(\'./other.json\', {\n            with: {\n                type: \'json\'\n            }\n        }));\n    });\n};\n'
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
