# Contributing to TypoMorph

## Approval and scope

Read [SPEC.md](docs/SPEC.md), [DECISIONS.md](docs/DECISIONS.md), and [OPEN_QUESTIONS.md](docs/OPEN_QUESTIONS.md). Requirements and revised documentation are approved. The [implementation plan](docs/IMPLEMENTATION_PLAN.md) is also approved; local implementation and controlled validation within its scope may proceed.

Do not use a documentation task as authorization to refactor, fix BUG-001, change deployment or legal terms, publish, or commit. Inspect Git state and preserve unrelated changes.

## Development setup

The current full workspace targets Linux. It uses Rust 2021, with `libdbus-1-dev` and `pkg-config` needed for the daemon's tray dependency. Live capture/replacement uses evdev/uinput; that path is not a portable platform abstraction.

```bash
cargo build --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

These are existing development commands, not commands executed during documentation work. See [TESTING.md](docs/TESTING.md) for evidence requirements. The current CI also runs a dependency audit against Cargo.lock.

Device-free checks can use synthetic input through [test-input](crates/daemon/README.md). Live `--dry-run` still reads devices. Current stderr diagnostics can contain text; do not run live checks on personal input or upload captured logs.

## Implementation conventions after approval

- Keep technical documentation, source identifiers, comments, and commit messages in English; user-facing strings follow the six-language UI requirement.
- Keep pure classification/mapping logic separate from OS integration and input persistence/networking.
- Explain non-obvious constraints and decisions in comments; prefer small, focused changes.
- Use synthetic fixtures and mocked transports for meaningful boundary and regression tests; no real credentials or payment operations in tests.
- Treat selected-layout mapping, protected/unknown fields, composition, races, and undo as explicit correctness boundaries.
- Preserve genuine input and fail safely when capabilities or context are uncertain.
- Keep release input out of logs, files, reports, and network requests. Development-only text diagnostics require an explicit separate boundary.
- Do not reintroduce obsolete AI, Free/Pro language gates, Enterprise scope, or periodic perpetual-license checks.
- Record evidence and uncertainty; a build or unit test is not proof of native application compatibility.

Existing test modules are commonly colocated with their code. Follow the surrounding style where appropriate; choose tests that exercise meaningful behavior rather than duplicating implementation.

## Validation and changes

Run checks appropriate to the authorized change and the CI requirements. For application changes, record focused regression and relevant real-platform evidence. For documentation-only changes, validate links, consistency, and scope without presenting that as runtime validation.

Describe the concrete problem, resulting behavior, validation, and remaining limitations in a PR. Keep commits focused when committing is authorized.

## Release preparation

Release work requires the later approved plan and verified readiness. The current Debian workflow uses the daemon package version; ensure any future release tag, Cargo version/lockfile, package name, website link, and actual artifact agree.

The landing page still has historical pricing/provider/platform claims. Its changes are outside the present documentation phase. Never infer a valid Windows/macOS package from a placeholder link.

Before publication, complete the [roadmap gates](docs/ROADMAP.md), validate actual package signatures and update behavior, and align public terms and privacy claims with demonstrated behavior. Stores and additional repositories are deferred.

## Bugs, security, and browser code

Record expected/actual behavior, synthetic reproduction, exact environment/build, severity, frequency, and evidence status. See [BUGS.md](docs/BUGS.md). Security reports follow [SECURITY.md](SECURITY.md), not a public issue containing sensitive data.

Browser extensions are conditional for the first release. Existing packaging and source status are documented in [extensions/README.md](extensions/README.md); neither packaging nor manual loading authorizes store publication.
