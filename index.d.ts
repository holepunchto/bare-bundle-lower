import Buffer from 'bare-buffer'
import Bundle from 'bare-bundle'

type LowerTarget =
  | 'es5'
  | 'es2015'
  | 'es2016'
  | 'es2017'
  | 'es2018'
  | 'es2019'
  | 'es2020'
  | 'es2021'
  | 'es2022'
  | 'es2023'
  | 'es2024'
  | 'esnext'

type LowerFormat = 'esm' | 'cjs'

interface LowerOptions {
  /**
   * The ECMAScript version to lower syntax to (default `'es5'`).
   */
  target?: LowerTarget
  /**
   * The module format to produce (default `'cjs'` for `es5` and `'esm'` otherwise). With `'cjs'`,
   * ES modules are converted to CommonJS and `import()` is converted to `require()` in every
   * module, for engines that support neither. `es5` only supports `'cjs'`.
   */
  format?: LowerFormat
}

interface LowerDiagnostic {
  url: string
  start: number
  end: number
  message: string
}

/**
 * Lower the syntax of the modules of `bundle` to `opts.target` and return a new bundle. Only syntax
 * is lowered; runtime APIs such as `Promise` and `Symbol` are assumed to be available. Each file is
 * handled according to the `type` recorded for it in the bundle header, or its extension if none is
 * recorded: `script` and `module` files are lowered and other files are left untouched. Files
 * listed as assets are always left untouched.
 * @throws {SyntaxError} a module cannot be parsed or lowered. The error has a `diagnostics` property
 * listing every problem found.
 */
declare function lower(bundle: Bundle | Buffer, opts?: LowerOptions): Bundle

declare namespace lower {
  export { type LowerTarget, type LowerFormat, type LowerOptions, type LowerDiagnostic }
}

export = lower
