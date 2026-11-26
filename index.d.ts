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

interface LowerOptions {
  /**
   * The ECMAScript version to lower syntax to (default `'es5'`). Targets before `es2015` don't
   * support ES modules, so ES modules are also converted to CommonJS.
   */
  target?: LowerTarget
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
  export { type LowerTarget, type LowerOptions, type LowerDiagnostic }
}

export = lower
