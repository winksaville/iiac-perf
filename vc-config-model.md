# vc-x1 config file

A model config file: every table and key a workspace config may carry, each with its default or a
typical value. Generated from the schema, so it cannot fall behind the keys the binary knows. Copy
the tables you need into your own `.vc-config.md` and drop the rest.

The values inside a `toml` fence are the workspace's own, and the prose around them is
documentation. Only a fence tagged exactly `toml` is read, so any other fence is prose like the
text beside it. Each bullet links to its key's entry in the schema documentation.

The `[agent-session]` table
- items: Default agent-session item set (comma-separated) [[1]]
- result-lines: Default --result-lines: max lines shown per tool result (0 = unlimited) [[2]]
- col-width: Default --col-width: first-column width in the --fields / --unknown / --per-line views
  [[3]]
```toml
[agent-session]
items = "headers,user,assistant,tool,summary"
result-lines = 10
col-width = 68
```

The `[repos]` table
- work: The work repo's path, relative to this config file's directory ("." in the work repo, ".."
  in the agent repo). The entry resolving to the config's own directory names the side [[4]]
- agent: The agent repo's path, relative to this config file's directory (e.g. ".claude" in the work
  repo, "." in the agent repo). Presence signals dual-repo mode [[5]]
```toml
[repos]
work = "."
agent = ".claude"
```

The `[family]` table
- member: This repo's member name in its agent-file family (also its record file in the messages
  repo) [[6]]
- template: Path to the family's template repository, relative to this config file's directory [[7]]
- messages: Path to the family's messages repo, relative to this config file's directory [[8]]
```toml
[family]
member = "vc-x1"
template = "../vc-x1-template"
messages = "../vc-x1-messages"
```

The `[validate]` table
- full: Full validation, in order, one invocation per element, run by `vc-x1 validate` [[9]]
- fast: Fast validation, in order, one invocation per element, run by `vc-x1 validate --fast` [[10]]
```toml
[validate]
full = [
  "cargo fmt",
  "cargo clippy --all-targets -- -D warnings",
  "cargo test",
  "cargo install --path . --locked",
]
fast = ["cargo test --bins"]
```

# References

[1]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#agent-sessionitems
[2]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#agent-sessionresult-lines
[3]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#agent-sessioncol-width
[4]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#reposwork
[5]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#reposagent
[6]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#familymember
[7]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#familytemplate
[8]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#familymessages
[9]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#validatefull
[10]: https://github.com/winksaville/vc-x1/blob/HEAD/vc-config.md#validatefast
