# mise

`ditch` is not in the built-in mise registry. Use the GitHub backend:

```bash
mise use -g github:baoulo/ditch@0.1.0
```

Or in `mise.toml`:

```toml
[tools]
"github:baoulo/ditch" = "0.1.0"
```

To keep a short name in `.tool-versions` (`ditch 0.1.0`):

```toml
[tool_alias]
ditch = "github:baoulo/ditch"
```

If multiple assets match, narrow with `matching = "ditch-"`.
