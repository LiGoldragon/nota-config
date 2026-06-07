# nota-config

Typed configuration input for Persona-stack binaries.

A binary's `main` reads its configuration through
`ConfigurationSource::from_argv()`, decodes into the binary's own
typed configuration record, and runs. Configuration arrives as
typed NOTA records — never as environment variables.
Production argv is exactly one argument: an inline NOTA record, a
`.nota` path, or a `.rkyv` path.

## Three transports

| Argv form | Transport |
|---|---|
| `"([we're ready] 64)"` | inline NOTA on argv |
| `path/to/config.nota` | NOTA file |
| `path/to/config.rkyv` | rkyv archive |

Detection is extension-based on that one argument: `(` prefix →
inline; `.nota` → NOTA file; `.rkyv` → rkyv file. No
content-sniffing. Inline NOTA must be quoted by the shell so it
arrives as one argv token.

## Usage

```rust
use nota_config::{ConfigurationRecord, ConfigurationSource, impl_rkyv_configuration};
use nota_next::{NotaDecode, NotaEncode};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

#[derive(NotaEncode, NotaDecode, Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq)]
pub struct MessageDaemonConfiguration {
    pub socket_path: String,
    pub backlog: u64,
}

impl_rkyv_configuration!(MessageDaemonConfiguration);

fn main() -> nota_config::Result<()> {
    let configuration: MessageDaemonConfiguration =
        ConfigurationSource::from_argv()?.decode()?;
    // … run with `configuration`
    Ok(())
}
```

For NOTA-only types (no rkyv archive support), invoke
`impl_nota_only_configuration!` instead.

## See

- `ARCHITECTURE.md` for module layout, detection algorithm,
  and the design rationale around the two-macro pattern.
- `reports/designer/183-typed-configuration-input-pattern.md`
  in the primary workspace for the full design.
