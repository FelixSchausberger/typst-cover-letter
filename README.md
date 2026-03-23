# typst-cover-letter

Personal cover letter template as a local [Typst](https://typst.app) package,
managed by a small Rust CLI.

## Setup

```sh
nix develop   # enters dev shell, auto-links @local/cover-letter:0.1.0
```

## Usage

### Create a new cover letter

```sh
coverletter new
```

Interactive wizard: company, address, contact person (optional), position,
language (de/en), date → opens `$VISUAL`/`$EDITOR`/`hx` → optionally compiles
to PDF.

Non-interactive:
```sh
coverletter new --company "Siemens AG" --position "Software Engineer" --lang de
```

### Build

```sh
coverletter build                        # compile .typ in current dir
coverletter build path/to/dir            # compile .typ in given dir
coverletter build --all ~/Fulltime       # compile all recursively (parallel)
coverletter build --all ~/Fulltime --force  # recompile even if PDF is newer
```

### Migrate (temporary)

One-time migration of existing letters from `@preview/appreciated-letter` to
`@local/cover-letter`:

```sh
coverletter migrate --dry-run ~/Fulltime   # preview changes
coverletter migrate ~/Fulltime             # apply
```

Delete `src/cmd/migrate.rs` and the `Migrate` arm in `src/cli.rs` after use.

## Package structure

The repo root is the Typst package (`typst.toml` + `lib.typ`).
`defaults.toml` holds the sender address — edit it when you move.

```
@local/cover-letter:0.1.0
  → symlinked to this repo by the dev shell shellHook
```

## Nix

```sh
nix build          # build coverletter binary
nix run .# -- new  # run without dev shell
```
