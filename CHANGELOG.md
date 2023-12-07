# 1.6.0

- Add the `--depth` option to limit many levels of dependencies are displayed
  starting from the root package(s)
- Upgrade dependencies

# 1.5.0

- Add the `--include` option to only shows given packages

# 1.4.0

- Add the `--root` option to specify a subset of a workspace's crates you want
  to get the dependency graph for
- Only include proc-macro crates in the graph if `--build-deps` is used

# 1.3.1

- Upgrade dependencies

# 1.3.0

- Add the `--workspace-only` option to get a graph of just the workspace
  packages
- Make the `--help` output prettier by upgrading to clap version `4.0.0-rc.2`

# 1.2.5

- Upgrade dependencies

# 1.2.4

- Fix some invalid handling of `cargo metadata` output that lead to inaccurate
  output and the warning

  ```
  crate from resolve not found in packages => dependencies
  ```

  on stderr

# 1.2.3

- Upgrade dependencies

# 1.2.2

- Fix `--exclude` not working for workspace members

# 1.2.1

- Calculate dependency kinds correctly in all cases
- Detect whether a crate is optional correctly in all cases
  - Previously, a crate with a hyphen in its crates.io name would never be shown
    as optional

# 1.2.0

- Rename `--exclude` to `--hide`
- Add a new `--exclude` option that is the same as `--hide`, except that it
  doesn't take the crate(s) in question into account for dependency kind
  resolution
- Improve handling of proc-macro crates in the workspace

# 1.1.2

- Fix excessive whitespace in option descriptions in `--help`

# 1.1.1

- Mark proc-macro crates in the workspace as build dependencies

# 1.1.0

- Add the `--focus` option
