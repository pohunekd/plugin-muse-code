# plugin-muse-code

Pohunek runtime plugin for [Meta's Muse Code](https://dev.meta.ai/docs/muse-code)
terminal coding agent.

| | |
|---|---|
| Repository | `pohunekd/plugin-muse-code` |
| Package id | `pohunek.runtime.muse` |
| Runtime id (`--agent`) | `muse` |
| Upstream | Meta Muse Code (proprietary CLI; the SDK and wire schema are MIT) |
| Status | **pre-release**: no descriptor yet, evidence only; fixtures are unverified until the release in `compat/RELEASE` is the release you run |

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
| `compat/RELEASE` | The Muse release every capture comes from |
| `compat/install/` | `--version`, install layout, binary sha256, the process the launcher leaves running |
| `compat/help/` | `--help` of every command |
| `compat/screens/<scenario>/<width>.txt` | Terminal screens at widths 20 to 200: trust, login, api-key, idle, working, done, approval, question, update-notice, clear, fork, resume, seed |
| `compat/osc7501/` | OSC 7501 program-status records per scenario, terminal titles, the startup queries |
| `compat/hooks/<scenario>/` | One JSON file per hook call: event, stdin payload, environment names, the `MUSE_*`/`TBH_*` values |
| `compat/macos-arm64/` | The same layout for macOS arm64: install, help, the login, api-key, trust, idle, working, done and update-notice screens, OSC 7501 records and titles, the default-HOME tree and the credential backend the sessions report |
| `compat/sessions/` | Session store tree and the first records of a session (strings over 80 characters become `<N chars>`; host fields such as pids and free disk space are dropped) |

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

Needs bash 4+, tmux, curl, jq, sed, awk, perl, sha256sum, git, ps, socat, ip, mount,
hostname and util-linux `unshare` 2.38+ with unprivileged user, network, mount
and UTS namespaces (`--map-root-user`, `--map-user`).

```bash
scripts/capture --root /path/to/scratch --dry-run all   # plan and tool check; no network
scripts/capture --root /path/to/scratch all             # install Muse there and capture everything
scripts/capture --root /path/to/scratch hooks stub      # single steps
```

`--root` receives the throwaway HOME (the CLI is installed into
`<root>/fakehome/.local/bin`), one XDG root per scenario and the raw terminal
streams. Run it on a disposable host or VM; never point it at a real HOME.
Paths below `--root` are replaced in the evidence by a placeholder of the same
width, so the layout of narrow screens is preserved.

On macOS the kit runs the screen and OSC steps (`install version login trust turn update-notice`; `hooks`, `sessions` and `stub` are refused) under a `sandbox-exec` profile, with bash 4+ and GNU sed, stat, find and date first on the PATH (for example from nix); `--out compat/macos-arm64` selects the evidence directory. The profile denies all socket traffic including loopback, allows unix sockets only below `--root` (tmux, at most 60 characters), denies the mach lookups that resolve names, reach the Keychain or hand a URL to the user's browser, and denies `/usr/bin/open`. The credential backend is forced to a file for every launcher; the sessions report it in `compat/macos-arm64/sessions/credential-backend-*.txt`, and the kit fails if a session reports another backend. Before the first step the kit takes controls outside the sandbox (a name resolves, a loopback connection works, a Keychain lookup answers) and requires each probe to fail inside it. Its scan also looks for the host name, computer name, full name, serial number and hardware UUID, which it reads at run time.

The profile does not confine the filesystem: the CLI can still read the real user's home and write below its `~/Library`, so a dedicated throwaway user or a VM would be stronger. The default-HOME tree on macOS lacks the `runtime/` and `session-server` entries of the Linux run.

A new Muse release is captured by re-running the kit before the lock moves:
`supported.below` stays at the next minor until the new captures pass.

## Verified behaviour

Every row comes from a capture of the release in `compat/RELEASE` on Linux
x86_64 in a throwaway HOME. "Stub" means the behaviour was observed against the
loopback model stub, not a real model. Behaviour without a capture is listed as
unverified.

| Area | Result |
|---|---|
| Version line | `muse --version` prints `Muse Code 1.4.14 (1.4.14-R7309.1)`; the release id is `X.Y.Z-R<n>.<m>` |
| Install | Launcher script `muse` plus `.muse-version`, `.muse-release-info.json` and the static binary `muse-bin-<release>`; the pane process is the binary, so `comm` is `muse-bin-<release>` truncated by the kernel |
| Trust dialog | `Do you trust this workspace?` with `1  Trust and continue` / `2  Quit`; `--trust-workspace` skips it |
| Login | Two entries, `Log in with browser` and `Set an API key`; the key prompt reads `Paste your Model API key.` |
| Idle | Composer prompt `❯`, footer `<model> · <cwd> · Auto-review`; the first idle frame, resume and seed screens of these captures also carry `system SSH configuration projection omitted: source ownership, mode or type does not qualify` (a capture artefact, see Unverified); `⟩` is the cursor of dialogs and pickers, not the idle prompt |
| Working | `◇`/`◈ Thinking (Ns · esc to interrupt)`, also `Calling tools`, `Double checking`, `Request user input <header> — running` (glyph alternates) |
| Approval (stub) | `Would you like to run the following command?` with `Allow this stage once (y)` and `Abort the entire command (esc)` |
| Question picker (stub) | `Enter to select · ↑/↓ to move · Tab for an optional note · Esc to interrupt` |
| OSC 7501 | Sent as `ESC ] 7501 ; ? ESC \` at startup; records follow only after the probe is answered with the same sequence, so the descriptor must answer it; `app=muse`; states `idle`, `working`, `done`, `blocked:kind=permission:app=muse`, `blocked:kind=question:app=muse`, `state=working:app=muse:id=term-<hex>` and `state=clear:id=term-<hex>` for background terminals; terminator `ESC \` |
| Terminal title | `OSC 0`: the workspace directory name when idle, prefixed by a braille spinner frame while working |
| Terminal modes | Bracketed paste, focus reporting and the kitty keyboard protocol are enabled; the CLI queries `OSC 10`, `OSC 11`, `OSC 4;0..15` and the cursor position at startup |
| Hooks | Project `.muse/hooks.json` (needs trust) and a managed file through `TBH_MANAGED_HOOKS_PATH` both run; project hooks were captured with `--trust-workspace` only |
| Hook environment | Cleared to `HOME`, `MUSE_SESSION_ID`, `MUSE_THREAD_ID`, `PATH`, `PWD`, `SHLVL`, `TERM` and, on most calls, `TRACEPARENT`; `-c 'managed_hooks_env_vars=["NAME"]'` adds names, without it they are dropped |
| Hook payload | `transcript_path` is always `null`; fields per event are in `compat/hooks/` |
| Session start hook | `SessionStart` `source` is `startup`, `clear`, `resume` or `fork` |
| Subagents | Background observers report `SubagentStart`/`SubagentStop` with `subagent_id` like `skill-reminder` and their own `session_id`; `MUSE_SESSION_ID` stays the root session |
| Session store | `$XDG_DATA_HOME/muse/sessions/YYYY/MM/DD/<uuid>/session.jsonl`, subagents below `subagent/<uuid>/`; the config home gets no `settings.json` |
| Resume and fork | `muse resume <id>` works for a root session and for a fork; `/fork` prints the fork id and `muse resume <id>`; `/clear` starts a new session and prints the old id |
| Prompt seed | `muse "<prompt>"` submits the prompt as the first turn |
| Model-free modes | `--provider echo` (credential-free) and `--base-url` against a loopback endpoint both work |

### macOS arm64

Captured on a macOS 27.0.1 arm64 host in a throwaway HOME under the sandbox profile above; evidence in `compat/macos-arm64/`.

| Area | Result |
|---|---|
| Release | Same release `1.4.14-R7309.1` and the same launcher (sha256 equal); the binary is a 423,687,488-byte Mach-O build |
| OSC 7501 | Identical to Linux: probe `ESC ] 7501 ; ? ESC \` at startup, records only after it is answered, `app=muse`, states `idle`, `working`, `done`; terminal title is the directory name with a braille spinner while working |
| Screens | Trust, login and api-key (not at width 20) screens match Linux. The idle, working and done composers carry an extra divider row `── Voice input (⌥ + v to start) ──` above `❯`, and the first idle frame has no SSH-projection notice |
| Launcher notice | `Muse Code updated A -> B` as on Linux |
| Credential backend | With `TBH_CREDENTIAL_BACKEND=file` the session record reports `credential_backend` `file` and `keychain_fallback_reason` `none`, in the default-HOME run and in the turn scenario |

Not captured on macOS: hooks, session resume and fork, the stub screens (no `socat`), the pane process (`ps` is refused inside the sandbox), and the default Keychain backend (the file backend was forced). The default-HOME tree (`sessions/default-home-tree.txt`) is captured. The rest stays unverified there.

### Unverified

- The first idle frame, the resume screens and the seed screen in a normal, non-namespaced run: unprivileged namespaces cannot map host root, so root-owned files appear owned by nobody and Muse prints the SSH-projection notice there. The footer itself is backed by `screens/done/<w>.txt` and `screens/working/<w>.txt` at every width.
- On macOS: hooks, session resume and fork, the stub screens, and the Keychain
  credential backend.
- A real model turn, real approval variants beyond the single-stage shell
  command (`Always allow in this workspace`), `Notification` for approvals,
  `PostToolUseFailure`, compaction hooks, `StopFailure`, `Interrupt`.
- When `SessionStart` is reported: the captures always include a turn, so reporting at launch versus with the first turn is not shown.
- That `Interrupt` and `ToolUseStart` hooks need `async: true` (a startup warning in an earlier exploration, not committed).
- The API-key prompt at width 20 (the wrapped login text defeats the kit's wait).
- Role subagents spawned by the model (only observer subagents were seen).
- Whether `TBH_MANAGED_HOOKS_PATH` overrides a `managed_hooks_path` set in
  `settings.json`.
- The terminal-side update banner driven by the release manifest (`urgency`,
  `notification_text`); the launcher's own update notice is captured.
- That `MUSE_NO_AUTO_UPDATE=1` leaves the network untouched.

Prior art cited by the plan (agent-deck, par-term, Zentty, Paradev, t3code,
openclaw) was not re-verified here.
