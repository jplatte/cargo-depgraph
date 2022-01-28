# [unreleased]

# 1.2.3

* Upgrade dependencies

# 1.2.2

* Fix `--exclude` not working for workspace members

# 1.2.1

* Calculate dependency kinds correctly in all cases
* Detect whether a crate is optional correctly in all cases
  * Previously, a crate with a hyphen in its crates.io name would never be shown
    as optional

# 1.2.0

* Rename `--exclude` to `--hide`
* Add a new `--exclude` option that is the same as `--hide`, except that it
  doesn't take the crate(s) in question into account for dependency kind
  resolution
* Improve handling of proc-macro crates in the workspace

# 1.1.2

* Fix excessive whitespace in option descriptions in `--help`

# 1.1.1

* Mark proc-macro crates in the workspace as build dependencies

# 1.1.0

* Add the `--focus` option
