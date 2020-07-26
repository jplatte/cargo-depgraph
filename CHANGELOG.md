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
