# plugin-muse-code

Pohunek runtime plugin for [Meta's Muse Code](https://dev.meta.ai/docs/muse-code)
terminal coding agent.

| | |
|---|---|
| Repository | `pohunekd/plugin-muse-code` |
| Package id | `pohunek.runtime.muse` |
| Runtime id (`--agent`) | `muse` |
| Upstream | Meta Muse Code (proprietary CLI; the SDK and wire schema are MIT) |
| Status | **pre-release**: no descriptor yet, evidence only; no evidence is committed yet |

The plugin carries data and evidence only (descriptor, detect rules, captures).
Handlers, reporters and parsers stay in the core repository
[`zajca/pohunek`](https://github.com/zajca/pohunek); the plan is the epic
`zajca/pohunek#1644` and the issues P0-P5 of this repository.

## What is here

| Path | Content |
|---|---|
| `scripts/capture` | Capture kit (bash, no third-party code): installs Muse into a throwaway HOME, drives it in a terminal and records what it shows |
| `scripts/hook-log` | Hook command that logs the stdin payload and the environment names of a hook call |
| `scripts/model-stub` | Loopback model endpoint (bash + jq) for approval and question screens without a model |

## No credentials, no real model

There is no Meta API key for this project and none will be created.
The kit never logs in, never stores a credential and never calls a real model:

- Model-free turns use Muse's own `--provider echo`.
- Every step except `install` runs in a network namespace that has only a
  loopback interface, so no capture can reach Meta.
- Approval and question screens use `scripts/model-stub`, a loopback endpoint
  reached through `--base-url`. The CLI gets a placeholder `META_API_KEY` for
  that run only; the kit fails if any credential file appears.
- The browser sign-in entry of the login screen is never selected: Enter on it
  starts a device-code request at Meta.

## Running the kit

Needs bash 4+, tmux, curl, jq, sed, awk, sha256sum, git, ps, socat, ip and
unprivileged user namespaces (`unshare --user --map-root-user --net`).

```bash
scripts/capture --root /path/to/scratch --dry-run all   # plan and tool check; no network
scripts/capture --root /path/to/scratch all             # install Muse there and capture everything
scripts/capture --root /path/to/scratch hooks stub      # single steps
```

`--root` receives the throwaway HOME (the CLI is installed into
`<root>/home/.local/bin`), one XDG root per scenario and the raw terminal
streams. Run it on a disposable host or VM; never point it at a real HOME.
Paths below `--root` are replaced in the evidence by a placeholder of the same
width, so the layout of narrow screens is preserved.

A new Muse release is captured by re-running the kit before the lock moves:
`supported.below` stays at the next minor until the new captures pass.

