# nota-config — architecture

## Role

`nota-config` is the **typed configuration input library** every
Persona-stack binary uses. A binary's `main` reads its configuration
through `ConfigurationSource::from_argv()`, decodes into the binary's
own typed configuration record, and runs. Environment variables are
**not** a control-plane configuration channel for production
binaries.

This file describes what the system **is** today (the shape that
ships). The design that motivates it is
`reports/designer/183-typed-configuration-input-pattern.md` in the
primary workspace.

## Boundaries

**Owns:**

- The `ConfigurationSource` enum — the three argv-derived transports
  (inline NOTA, NOTA file, rkyv file).
- `ConfigurationSource::from_argv()` / `from_args()` — strict
  one-argument argv parsing and extension-based detection.
- `ConfigurationSource::decode<C>()` — dispatch into a typed
  configuration record.
- The `ConfigurationRecord` trait — extends `NotaDecode + Sized`
  with `from_rkyv_bytes`.
- The `impl_nota_only_configuration!` and `impl_rkyv_configuration!`
  macros — installer macros for the two halves of the trait.
- The crate-wide `Error` enum.

**Does not own:**

- The configuration records themselves — each consumer crate (e.g.
  `signal-persona-message`) defines its own
  `<X>DaemonConfiguration` record.
- The NOTA codec — that lives in `nota-codec`. This crate re-uses
  `Decoder`, `NotaDecode`, and the derive macros from there.
- The rkyv codec — that lives in the `rkyv` crate. This crate calls
  `rkyv::from_bytes` and converts the error.

## Three transports, one-argument dispatch

| Argv form | Transport | When |
|---|---|---|
| `"([we're ready] High)"` | inline NOTA on argv | small configs, debugging, ad-hoc daemon launches |
| `path/to/config.nota` | NOTA file | larger configs, human-readable, production today |
| `path/to/config.rkyv` | rkyv archive | future hot-path, pre-validated, rkyv-derived types |

Detection rules, in priority order:

0. **Arg count must be exactly one.** No argument returns
   `MissingArgument`; more than one returns `MultipleArguments`.
1. **The argument starts with `(`** → inline NOTA. The shell must
   quote the whole record so it arrives as one argv token.
2. **The argument ends with `.nota`** → NOTA file.
3. **The argument ends with `.rkyv`** → rkyv file.
4. Anything else → typed `Error`.

No content-sniffing of file bodies. Filenames self-document their
format.

## Two macros, no blanket impl

`impl_nota_only_configuration!` installs a `ConfigurationRecord`
impl whose `from_rkyv_bytes` returns `Error::RkyvNotSupported`.
`impl_rkyv_configuration!` installs an impl that decodes through
`rkyv::from_bytes`. **Every configuration record invokes exactly
one of the two.**

This is a deliberate deviation from `designer/183` §3.1, which
proposed a blanket `impl<T: NotaDecode> ConfigurationRecord for T`
plus a per-type `impl_rkyv_configuration!` override. Rust forbids
overlapping impls — the blanket already covers any `T: NotaDecode`,
so a per-type re-impl conflicts at compile time. (Resolving the
overlap properly would require unstable specialization.)

The two-macro shape preserves the design's intent — explicit opt-in
per type, NOTA-only is the default-shaped case — without requiring
specialization, and reads symmetrically at the call site:

```rust
impl_nota_only_configuration!(SimpleConfig);     // NOTA-only
impl_rkyv_configuration!(MessageDaemonConfiguration);  // dual NOTA + rkyv
```

The cost is one one-line macro invocation per configuration type;
the gain is sound Rust.

## Test-shim discipline

Production binaries call `ConfigurationSource::from_argv()`. Test
shims may call `from_argv_with_test_env_fallback(env_var_name)`,
which falls back to the named environment variable **only** when
argv has no configuration argument. The method is `#[doc(hidden)]`
and carries `test_env_fallback` in the name so:

- Anyone reading a binary's `main` sees the relaxation.
- Code review can grep for `with_test_env_fallback` and flag any
  production binary that uses it.
- The env-var name is explicit per binary (no ambient
  `NOTA_CONFIG` default).

The testable kernel is `from_args_with_env_fallback(args, env)`,
which takes both the argv iterator and the env-var value as
parameters. The `_argv_` variant is a thin wrapper that pulls
`std::env::args_os()` and `std::env::var_os(...)` and delegates.

## Code map

```
src/
├── lib.rs           # re-exports, macros, crate-level doc
├── error.rs         # Error enum + Result<T> type alias
├── source.rs        # ConfigurationSource enum, from_args/from_argv/decode
└── configuration.rs # ConfigurationRecord trait

tests/
├── argv_detection.rs   # '(' / .nota / .rkyv detection, error paths
├── inline_nota.rs      # round-trip an inline NOTA argv into a typed record
├── nota_file.rs        # round-trip a .nota file into a typed record
├── rkyv_file.rs        # round-trip a .rkyv file into a typed record; nota-only type rejects
└── test_env_fallback.rs # env-var fallback works only via opt-in
```

## Cross-cutting context

- Used by every Persona-stack daemon (`persona-message-daemon`,
  `persona-router-daemon`, …) and by the new `lojix` daemon + CLI.
- Per-component contract crates own their typed
  `<X>DaemonConfiguration` records (per `designer/183` §8 Q4); this
  crate provides the shared decode/dispatch surface, not the
  records themselves.
- The rkyv feature set pinned here
  (`std + bytecheck + little_endian + pointer_width_32 + unaligned`)
  matches every other Persona-stack contract crate; archives are
  interoperable across the fabric.

## Status

**Working core.** Public surface supports strict one-argument
configuration input, NOTA-only and rkyv-capable configuration
records, and the test-only environment fallback. The two-macro
shape is the intentional deviation from `designer/183` §3; the
single-argument enforcement keeps this crate aligned with the
component-binary argv contract.

## Macro-pattern integration

**Status:** integrated into the brilliant macro library pattern per `reports/designer/326-v13-spirit-complete-schema-vision.md §3` (schemas as macro-pattern instance).

**Role:** this crate is the NOTA config helper — the single-argument NOTA-string config decoder every component binary uses to consume its CLI argument. It is orthogonal to the schema-engine upgrade; it sits at the binary's argv boundary, not inside the wire protocol.

**Integration target:** NOTA notation suite; config records remain hand-declared per component and continue to derive `NotaRecord` via `nota-derive`. The schema-engine upgrade does not touch this crate's surface, but the macro-emitted config types (where the schema declares a component-config block) plug into this crate's decode pipeline through the same `nota-derive` derives.

**Per-library concern:** if a future schema-language extension adds a config-block declaration (sketch only, not in `/326-v13`), nota-config remains the runtime decoder; the macro would just emit the same shape the human author writes today.

**References:**
- `reports/designer/326-v13-spirit-complete-schema-vision.md` — schema language + macro pattern
- `reports/designer/324-migration-mvp-spirit-handover-re-specification.md` — migration MVP
- `reports/operator/174-schema-import-header-design-critique-2026-05-24.md` — lowering + AssembledSchema form
