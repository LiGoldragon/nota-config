You **MUST** read AGENTS.md at `github:LiGoldragon/lore` — the canonical agent contract.

## Repo role

`nota-config` is the **typed configuration input library** every
Persona-stack binary uses. A binary's `main` reads its configuration
through `ConfigurationSource::from_argv()`, decodes into the binary's
own typed configuration record, and runs. Environment variables are
**not** a control-plane configuration channel; production binaries
read exactly one configuration argument from argv only.

Three transports, one dispatch:

| Argv form | Transport | When |
|---|---|---|
| `'([/run/persona/spirit.sock] 64)'` | inline NOTA on argv | small configs, debugging, ad-hoc daemon launches |
| `path/to/config.nota` | NOTA file | larger configs, human-readable, production today |
| `path/to/config.rkyv` | rkyv archive | future hot-path, pre-validated, rkyv-derived types |

Detection is **extension-based**: `(` prefix → inline NOTA; `.nota`
suffix → NOTA file; `.rkyv` suffix → rkyv file. No content-sniffing.
Inline NOTA must be shell-quoted so it arrives as one argv token;
multiple production argv tokens are a typed error, not a joinable
record fragment.

## Carve-outs worth knowing

- **No environment variables in production paths.** The only env-var
  surface is `ConfigurationSource::from_argv_with_test_env_fallback`,
  which is `#[doc(hidden)]` and named so reviewers can grep for it.
- **Two macros, no blanket impl.** `impl_nota_only_configuration!`
  installs a rkyv-rejecting impl; `impl_rkyv_configuration!` installs
  a real rkyv decoder. Every configuration record invokes exactly
  one. This avoids the blanket+macro overlap that Rust forbids.
- **Tests live in separate files** under `tests/` — one file per
  detection / decode shape.

## Designer pointer

The design that motivates this crate is `reports/designer/183-typed-configuration-input-pattern.md`
in the primary workspace. Read it before changing the public surface.
