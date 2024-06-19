# @ayaseaki/swc-plugin-monorepo-relative-path-resolve

A SWC plugin to handle resolving relative import paths like `@/*` and `@common/*` of sub-packages in a monorepo.

## Why

For monorepo scene, we often have the need to directly import the source code of sub-packages to get a better bundle speed. 

However, when a sub-package in a monorepo has its own path alias defined in its `tsconfig.json`, the bundle of base app may not respect the alias config, which results in a failure of resolution:

```ts
// sub-package/src/index.ts
import { foo } from '@/common/constants';
export { foo };

// sub-package/src/common/constants.ts
export const foo = 'foo';
```

```ts
// base/index.ts
import { foo } from 'sub-package'; 
console.log(foo);

// while compiling base/index.ts `Can't resolve @/common/constants` will be thrown
```

This plugin aims to resolve a subset of this issue: if your alias is just a relative of `src` path, you can use this swc plugin to dynamically replace the import alias during compile-time.

## Usage

```bash
pnpm i -D @ayaseaki/swc-plugin-monorepo-relative-path-resolve
```

This plugin can handle the import source matching `{prefix}{subpath}?/`. For example:

- `@/*`
- `@common/*`
- `~common/*`

Usage with rspack:

```js
// in rspack.config.js (ts):
{
  test: /\.(jsx?|tsx?)$/,
  use: [
    {
      loader: "builtin:swc-loader",
      options: {
        jsc: {
          // ...
          // add the following config:
          experimental: {
            plugins: [
              [
                "@ayaseaki/swc-plugin-monorepo-relative-path-resolve",
                {
                  prefix: "@",
                  allowedPathnames: ["common"],
                },
              ],
            ],
          },
        },
      },
    },
  ],
},
```

See [example](https://github.com/kirainmoe/swc-plugin-monorepo-relative-path-resolve/tree/master/example) for detail.

## Configration

### prefix

- type: `string`
- description: specify the import prefix of your code.
- default: `@`

### allowedPathnames

- type: `string[]`
- description: specify the allowed pathnames after prefix, for example if you have a relative path `@common/` and `@components/`, then you set `allowedPathnames` to `['common', 'components'.]`
- default: `[]`
