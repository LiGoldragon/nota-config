# INTENT — nota-config

*What the psyche wants this project to be, and its most important
design constraints. Synthesised from workspace-backed intent that
applies to this repo plus the repo's specific purpose. Backed by the
repo's real purpose and applicable workspace constraints; not
embellished. Companion to `ARCHITECTURE.md`. Maintenance:
`primary/skills/repo-intent.md`.*

## Why this repo exists

`nota-config` is the **typed configuration-input library** every
component binary uses to read its single startup argument. A binary's
`main` calls `ConfigurationSource::from_argv()`, decodes into the
binary's own typed configuration record, and runs. This crate provides
the shared decode/dispatch surface; each consumer crate owns its own
`<X>DaemonConfiguration` record.

## Load-bearing constraints

*Every component binary takes exactly one argument — a NOTA string, a
`.nota` path, or a signal-encoded `.rkyv` path.* This crate IS the
enforcement of that workspace rule at the argv boundary. Argument count
must be exactly one: zero returns `MissingArgument`, more than one
returns `MultipleArguments`. There are no flags — not `--verbose`, not
`--config=path`, not anything. If a binary needs new configuration, the
configuration record's NOTA schema gets a new field, never a new flag.

*Environment variables are NOT a control-plane configuration channel
for production binaries.* Configuration arrives as one typed argument.
The only relaxation is the explicitly-named, `#[doc(hidden)]`
test-shim fallback (`from_argv_with_test_env_fallback`), which is
greppable by name so review can flag any production binary that reaches
for it.

*Transport detection is extension-based on the one argument, never
content-sniffing.* A `(` prefix is inline NOTA; a `.nota` suffix is a
NOTA file; a `.rkyv` suffix is a signal-encoded archive. Filenames
self-document their format; file bodies are never inspected to guess.

*NOTA strings come exclusively from bracket forms.* Inline NOTA on
argv uses bracket string forms (`[text]`, `[|text|]`) and bare
camelCase/kebab-case at `String` positions; the inline record is
wrapped in shell double quotes because NOTA itself never contains a
quotation mark. This crate decodes that text through `nota-next`; it
does not accept a second config syntax.

*This crate is library-only and owns the boundary, not the records.*
No daemon, no socket, no durable store. It owns `ConfigurationSource`,
the one-argument parse, the `ConfigurationRecord` trait, and the two
installer macros (`impl_nota_only_configuration!` /
`impl_rkyv_configuration!`). It does not own the configuration records
themselves, the NOTA codec (that is `nota-next`), or the rkyv codec.

## Schema-stack direction

This crate sits at the binary's argv boundary, orthogonal to the
schema-engine upgrade. Config records remain hand-declared per
component and derive `NotaEncode` / `NotaDecode` via `nota-next`;
where a future schema declares a component-config block, the
macro-emitted config type plugs into this same decode pipeline through
the same derives. The crate's surface does not change as the schema
engine lands.

## See also

- `ARCHITECTURE.md` — module layout, the three-transport detection
  algorithm, and the two-macro pattern.
- `primary/skills/component-triad.md` §"The single argument rule" —
  the workspace rule this crate enforces.
- `primary/skills/nota-design.md` — NOTA bracket-form string discipline.
