# CLAUDE.md

Developer notes for `github.com/tetratelabs/built-on-envoy`.

## CLI build
- Build the `boe` CLI from `cli/`: `make build` (runs codegen via `gen-code` first, then builds `out/boe-<goos>-<goarch>`). Generated code is committed, so on a box without codegen tooling you can skip codegen with a plain `go build -o out/boe .` from `cli/`.

## `boe run` with dynamic modules (Rust extensions)
- Dynamic modules (Rust cdylib) are loaded only by a **Linux** Envoy. On macOS the native build is a Mach-O `.dylib` that Envoy cannot `dlopen` — build/run on Linux (or use `--docker`).
- The native (func-e) run path does NOT require Docker on Linux. `boe run` with `--local <ext-dir>` will `cargo build --release` the module itself (`internal/extensions/dynamic_module.go`), then copy `target/release/lib<crate_name>.so` into the local cache as `lib<manifest.Name>.so` — i.e. the cargo underscore name is renamed to the manifest's (possibly hyphenated) name automatically. It then symlinks that into a temp dir and sets `ENVOY_DYNAMIC_MODULES_SEARCH_PATH` (`internal/envoy/runner.go: setupDynamicModuleSearchPath`).
- A Rust extension that implements `network` and/or `udp_listener` filters MUST declare `filterType:` in its `manifest.yaml` — the default is `[http]`, so omitting it means none of its filters get wired.
- All dynamic-module filter dispatch in the extension's Rust `lib.rs` keys on the **manifest `name`** (e.g. `"dns-gateway"`), not the crate/module name — the CLI emits `filter_name = manifest.Name` for every generated filter. Mismatch causes a runtime "Unknown filter name" panic.
- `--config '<json>'` is passed to the module verbatim as a `google.protobuf.StringValue` — the module should parse the raw string directly (e.g. serde into its config struct). Do not expect a protobuf-Struct `Any` envelope / `value` unwrap.
- To wire multiple filter types from one extension, list the extension once per filter with positional `--filter-type` flags, e.g. `--local <dir> --filter-type udp_listener --config '{...}' --local <dir> --filter-type network --config '{}'`.

## Envoy version resolution + dynamic-module proto compatibility
- The Envoy version func-e downloads is resolved from the extension manifest's `minEnvoyVersion` (`ResolveMinimumCompatibleEnvoyVersion`, `cli/cmd/run.go`). Override per-run with `--envoy-version <x.y.z>`.
- The `go-control-plane/envoy` proto bindings (see `go.mod`, pinned to a `v1.37.1-0.<pseudo>` post-release version) emit `DynamicModuleConfig.metrics_namespace` ("builtonenvoy", set in `internal/envoy/extension.go`). This field does NOT exist in the released Envoy **1.37.1** binary, so running against 1.37.1 fails bootstrap validation with `no such field: 'metrics_namespace'`. Use Envoy **1.38.0+** at runtime. (Any extension pinned to `minEnvoyVersion: 1.37.1` is affected.)

## Building Rust extensions natively
- `cargo build --release` for these extensions needs **libclang** at build time (transitive `bindgen` dep). On Debian/Ubuntu: `apt-get install -y clang libclang-dev`. Without it the build panics with "Unable to find libclang ... set the LIBCLANG_PATH environment variable".
- Verify a built `.so` is the right artifact with `file target/release/lib<name>.so` — expect `ELF ... shared object` for the target arch (x86-64 or ARM aarch64), NOT Mach-O.

## Local verification of the dns-gateway extension (example)
- `boe run --envoy-version 1.38.0 --local extensions/dns-gateway --filter-type udp_listener --config '{"domains":[{"domain":"*.aws.com","base_ip":"10.239.0.0","prefix_len":24,"metadata":{"cluster":"aws_cluster"}}]}'` stands up a UDP DNS listener on the listen port (default 10000).
- Test: `dig @127.0.0.1 -p 10000 s3.aws.com +short` returns a virtual IP (`10.239.0.0`), and a second distinct matching domain returns the next sequential IP (`10.239.0.1`).
