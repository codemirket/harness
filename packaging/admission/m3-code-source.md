# TypeScript source-parser runtime selection

The first-release JS/TS/JSX/TSX **structure** route uses the unchanged
`typescript@5.9.3` compiler API on Mirket's already private Node 24.22.1.
The same selected compiler now also powers the bounded
[semantic navigation route](https://github.com/codemirket/harness/blob/b3068a181a674f2aff751de4ff254f8120e7f96e/docs/m3-typescript-semantic-navigation.md).
The former TypeScript 7 analyzer candidate is not selected.
The exact npm lock entry has no registry runtime dependencies;
moving it from development-only to production changes only its lockfile
classification, not the archive pin. The npm integrity string remains
`sha512-jl1vZzPDinLr9eUt3J/t7V6FgNEw9QjvBPdysz9KfQDD41fQrC2Y4vKQdiaUpFT4bXlb1RHhLpp8wtm6M5TgSw==`.

Only these unchanged archive members are selected under
`adapters/node_modules/typescript/`:

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `package.json` | 3,620 | `822ef7ca6452205657b6288b066481ecf508bfbf43455d715cf7d3ec457561e6` |
| `lib/typescript.js` | 9,112,572 | `3ae902c92cc44dace175c0e69e13a4b0899f6983c6121d76b9ab8dd5795e7675` |
| `LICENSE.txt` | 9,197 | `a7d00bfd54525bc694b6e32f64c7ebcf5e6b7ae3657be5cc12767bce74654a47` |
| `ThirdPartyNoticeText.txt` | 37,824 | `1af3c68039c57e539422da82a4faada506ce6d0ea6f90e0b699d02dbcdb7a90c` |

The selected 9,163,213 bytes include Apache-2.0 terms and complete packaged
third-party notices. The source archive, integrity and passive probe evidence
are in [the TypeScript qualification](https://github.com/codemirket/harness/blob/b3068a181a674f2aff751de4ff254f8120e7f96e/docs/m3-typescript-navigation-qualification.md)
and its retained task-state inventory. No `tsc`, `tsserver`, default library,
npm/npx or TypeScript binary is selected. The release assembler rejects any
additional or altered TypeScript member; the release manifest pins every
selected file. The fixed package import resolves only within the verified
private release tree, not a project or host installation.

On 2026-10-25, `npm audit --omit=dev --package-lock-only` against this exact
adapter lock returned zero reported vulnerabilities for its production npm
packages; the JSON response is retained in task-state. This dated registry
check is not a proof that the parser or the complete binary distribution is
free of vulnerabilities.

The permitted API is `createSourceFile` plus `forEachChild` on supplied text
with an explicit JS/TS/JSX/TSX `ScriptKind`. No project configuration, host
filesystem, module resolution, program creation, emit, plugin, source execution
or automatic installation belongs to this route. The finite worker and MCP
integration must prove its own byte spans, revisions, budgets and cancellation
before the route is called installed or release-ready. This selection is not
semantic definition/reference support and does not waive Windows qualification.
