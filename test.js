const test = require('brittle')
const acorn = require('acorn')
const Bundle = require('bare-bundle')
const compile = require('bare-bundle-compile')
const lower = require('.')

test('lowered bundle is ES5 and evaluates to the same exports', async (t) => {
  const bundle = new Bundle()
    .write(
      '/main.mjs',
      `
      import greet, { Counter } from './counter.mjs'
      import values from './values.cjs'
      const { first, ...rest } = values
      const counter = new Counter(first)
      for (const value of rest.others) counter.add(value)
      function* doubled() { for (const value of [first, ...rest.others]) yield value * 2 }
      export const total = counter.total
      export const all = [...doubled()]
      export const greeting = greet\`bare\`
      export const missing = values.missing?.deep ?? 'none'
      export const later = (async () => (await Promise.resolve(counter)).total)()
      `,
      {
        main: true,
        imports: { './counter.mjs': '/counter.mjs', './values.cjs': '/values.cjs' }
      }
    )
    .write(
      '/counter.mjs',
      `
      export class Counter {
        #total = 0
        static zero = 0
        constructor(start = Counter.zero) { this.#total = start }
        add(value) { this.#total += value }
        get total() { return this.#total }
      }
      export default (strings) => \`hello \${strings[0]}\`
      `
    )
    .write('/values.cjs', `module.exports = { first: 1, others: [2, 3] }`)

  const lowered = lower(bundle)

  for (const [url, source] of lowered) {
    t.execution(
      () => acorn.parse(source.toString(), { ecmaVersion: 5, allowReturnOutsideFunction: true }),
      url
    )
  }

  const { exports } = eval(compile(lowered))

  t.is(exports.total, 6)
  t.alike(exports.all, [2, 4, 6])
  t.is(exports.greeting, 'hello bare')
  t.is(exports.missing, 'none')
  t.is(await exports.later, 6)
})

test('ES modules are converted to CommonJS only with the cjs format', (t) => {
  const bundle = new Bundle().write('/main.mjs', 'export const value = a ?? b', {
    main: true
  })

  const es5 = lower(bundle, { target: 'es5' })
  const es2015 = lower(bundle, { target: 'es2015' })
  const es2022 = lower(bundle, { target: 'es2022', format: 'cjs' })

  t.is(es5.type('/main.mjs'), 'script')
  t.is(es2015.type('/main.mjs'), null)
  t.is(es2022.type('/main.mjs'), 'script')
  t.snapshot(sources(es5))
  t.snapshot(sources(es2015))
  t.snapshot(sources(es2022))
})

test('import() is converted to require() in every module with the cjs format', async (t) => {
  const bundle = new Bundle()
    .write(
      '/main.cjs',
      `
      module.exports = {
        required: require('required'),
        imported: Promise.all([import('imported'), import('./module.mjs')])
      }
      `,
      {
        main: true,
        imports: {
          required: { require: '/dep.cjs', import: '/wrong.cjs' },
          imported: { import: '/dep.cjs', default: '/dep.cjs' },
          './module.mjs': '/module.mjs'
        }
      }
    )
    .write('/module.mjs', "export const value = () => import('./dep.cjs')", {
      imports: { './dep.cjs': '/dep.cjs' }
    })
    .write('/dep.cjs', 'module.exports = 1')
    .write('/wrong.cjs', 'module.exports = 0')

  const lowered = lower(bundle, { target: 'esnext', format: 'cjs' })

  t.is(lowered.type('/main.cjs'), 'script')
  t.alike(lowered.resolutions['/main.cjs'], {
    required: { require: '/dep.cjs', import: '/wrong.cjs' },
    imported: { require: '/dep.cjs', default: '/dep.cjs' },
    './module.mjs': '/module.mjs'
  })

  const { exports } = eval(compile(lowered))

  t.is(exports.required, 1)

  const [imported, module] = await exports.imported

  t.is(imported.default, 1)
  t.is((await module.value()).default, 1)
})

test('import attributes are passed on to require() when converting to CommonJS', (t) => {
  const bundle = new Bundle().write(
    '/main.mjs',
    `
    import text from './file.txt' with { type: 'text' }
    export * from './data.json' with { type: 'json' }
    export { default as bytes } from './file.bin' with { type: 'binary' }
    export const lazy = () => import('./other.json', { with: { type: 'json' } })
    export { text }
    `,
    { main: true }
  )

  t.snapshot(sources(lower(bundle)))
})

test('converted modules resolve their imports with the require condition', (t) => {
  const bundle = new Bundle()
    .write('/main.mjs', "import dep from 'dep'\nexport default dep", {
      main: true,
      imports: {
        dep: { require: '/dep.cjs', bare: { import: '/dep.mjs', default: '/dep.cjs' } }
      }
    })
    .write('/dep.mjs', 'export default 1')
    .write('/dep.cjs', 'module.exports = 2')

  t.alike(lower(bundle).resolutions, {
    '/main.mjs': { dep: { bare: { require: '/dep.mjs', default: '/dep.cjs' } } }
  })
})

test('imports that resolve to a different module for require() cannot be lowered', (t) => {
  const lowerWith = (dep) =>
    lower(
      new Bundle()
        .write('/main.cjs', "module.exports = [require('dep'), import('dep')]", {
          main: true,
          imports: { dep }
        })
        .write('/dep.mjs', 'export default 1')
        .write('/dep.cjs', 'module.exports = 1'),
      { target: 'esnext', format: 'cjs' }
    )

  const conflicts = [
    { require: '/dep.cjs', import: '/dep.mjs' },
    { import: '/dep.mjs', default: '/dep.cjs' },
    { bare: { import: '/dep.mjs', require: '/dep.cjs' } }
  ]

  for (const dep of conflicts) {
    t.exception.all(
      () => lowerWith(dep),
      /Cannot lower import of 'dep' as it resolves to a different module for require\(\) at \/main\.cjs:1:42$/
    )
  }

  const agreements = [
    '/dep.cjs',
    { import: '/dep.cjs', require: '/dep.cjs' },
    { import: '/dep.cjs', default: '/dep.cjs' },
    { bare: { import: '/dep.cjs', require: '/dep.cjs' }, default: '/dep.mjs' }
  ]

  for (const dep of agreements) t.execution(() => lowerWith(dep))
})

test('import.meta is rewritten to the CommonJS bindings of Bare', (t) => {
  const bundle = new Bundle().write(
    '/main.mjs',
    `
    export default {
      url: import.meta.url,
      main: import.meta.main,
      cache: import.meta.cache,
      dirname: import.meta.dirname,
      filename: import.meta.filename.length,
      resolve: import.meta.resolve('./dep'),
      addon: import.meta.addon(),
      host: import.meta.addon.host,
      addonPath: import.meta.addon.resolve(),
      asset: import.meta.asset('./file'),
      meta: import.meta,
      unknown: import.meta.unknown
    }
    `,
    { main: true }
  )

  t.snapshot(sources(lower(bundle)))
})

test('top-level await cannot be converted to CommonJS', (t) => {
  const bundle = new Bundle().write(
    '/main.mjs',
    `
    const value = await Promise.resolve(1)
    for await (const item of []) {}
    export default async () => await value
    `,
    { main: true }
  )

  try {
    lower(bundle)
    t.fail('should throw')
  } catch (err) {
    t.ok(err instanceof SyntaxError)
    t.snapshot(err.diagnostics)
  }

  t.execution(() => lower(bundle, { target: 'es2022' }))
})

test('syntax errors report the module and position', (t) => {
  const bundle = new Bundle()
    .write('/main.js', 'module.exports = 1', { main: true })
    .write('/broken.js', 'const a = 1\nconst = 2\n')

  try {
    lower(bundle)
    t.fail('should throw')
  } catch (err) {
    t.ok(err instanceof SyntaxError)
    t.snapshot(err.message)
    t.snapshot(err.diagnostics)
  }
})

test('assets and files that are not modules are untouched', (t) => {
  const asset = 'const untouched = () => 1'
  const json = '{ "kept": true }\n'
  const text = 'const kept = () => 1'

  const bundle = new Bundle()
    .write('/main.js', 'module.exports = 1', { main: true })
    .write('/data.json', json)
    .write('/asset.js', asset, { asset: true })
    .write('/template.js', text, { type: 'text' })

  const lowered = lower(bundle)

  t.is(lowered.read('/data.json').toString(), json)
  t.is(lowered.read('/asset.js').toString(), asset)
  t.is(lowered.read('/template.js').toString(), text)
  t.is(lowered.type('/template.js'), 'text')
})

test('unknown targets and formats are rejected', (t) => {
  const bundle = new Bundle().write('/main.js', 'module.exports = 1', { main: true })

  t.exception.all(() => lower(bundle, { target: 'es4' }), TypeError)
  t.exception.all(() => lower(bundle, { format: 'amd' }), TypeError)
  t.exception.all(() => lower(bundle, { target: 'es5', format: 'esm' }), TypeError)
})

function sources(bundle) {
  return Object.fromEntries([...bundle].map(([key, data]) => [key, data.toString()]))
}
