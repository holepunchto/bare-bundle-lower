const Bundle = require('bare-bundle')
const binding = require('#binding')

const targets = new Set([
  'es5',
  'es2015',
  'es2016',
  'es2017',
  'es2018',
  'es2019',
  'es2020',
  'es2021',
  'es2022',
  'es2023',
  'es2024',
  'esnext'
])

const formats = new Set(['esm', 'cjs'])

module.exports = function lower(bundle, opts = {}) {
  const { target = 'es5', format = target === 'es5' ? 'cjs' : 'esm' } = opts

  if (!targets.has(target)) throw new TypeError(`Unknown target '${target}'`)

  if (!formats.has(format)) throw new TypeError(`Unknown format '${format}'`)

  if (target === 'es5' && format === 'esm') {
    throw new TypeError(`Target '${target}' does not support ES modules`)
  }

  if (!Bundle.isBundle(bundle)) bundle = Bundle.from(bundle)

  const input = bundle.toBuffer()

  const result = binding.lower(input.buffer, input.byteOffset, input.byteLength, target, format)

  if (result.diagnostics) throw createError(bundle, result.diagnostics)

  return Bundle.from(Buffer.from(result.buffer))
}

function createError(bundle, diagnostics) {
  const [first] = diagnostics

  let message = first.message

  if (first.url) {
    const { line, column } = position(bundle.read(first.url), first.start)

    message += ` at ${first.url}:${line}:${column}`
  }

  if (diagnostics.length > 1) message += ` (and ${diagnostics.length - 1} more)`

  const err = new SyntaxError(message)
  err.diagnostics = diagnostics
  return err
}

function position(source, offset) {
  let line = 1
  let column = 1

  for (let i = 0; i < offset && i < source.byteLength; i++) {
    if (source[i] === 0x0a) {
      line++
      column = 1
    } else {
      column++
    }
  }

  return { line, column }
}
