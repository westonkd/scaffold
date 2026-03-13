# Blueprint Schema

## JSON Schema

```json
{
  "name": "string",
  "dependencies": [
    {
      "name": "string",
      "source": "string (git remote URL)",
      "ref": "string (branch / tag / commit, optional)",
      "path": "string (location inside parent repo to symlink into, optional)",
      "dependencies": []
    }
  ]
}
```

## Field Reference

| Field | Required | Description |
|---|---|---|
| `name` | yes | Workspace directory name (top-level) or repo directory name and symlink name (dependency) |
| `source` | yes | Git remote URL |
| `ref` | no | Branch, tag, or commit to check out (defaults to the remote default branch) |
| `path` | no | Relative path inside the parent repo where this dependency is symlinked |
| `dependencies` | no | Nested sub-dependencies (recursive) |

## Example: `koa-dev.json`

```json
{
  "name": "koa-dev",
  "dependencies": [
    {
      "name": "koa",
      "source": "https://github.com/koajs/koa.git",
      "ref": "master",
      "dependencies": [
        {
          "name": "router",
          "source": "https://github.com/koajs/router.git",
          "path": "packages",
          "ref": "master"
        }
      ]
    },
    {
      "name": "compose",
      "source": "https://github.com/koajs/compose.git",
      "ref": "master"
    }
  ]
}
```

Running `scaffold build koa-dev.json` with this blueprint will:

1. Clone (or update) each repo into `~/.scaffold/projects/`
2. Create local clones under `./koa-dev/repos/`
3. Symlink `koa` and `compose` at the `koa-dev/` root
4. Symlink `router` into `koa/packages/` so it appears as a local package
5. Copy `koa-dev.json` into `koa-dev/blueprint.json`

## Notes

- The blueprint file is copied into the workspace as `blueprint.json` at build time.
- `scaffold update` reads from `./blueprint.json` inside the workspace, not the original blueprint file. Edit `blueprint.json` inside the workspace to change what `update` does.
