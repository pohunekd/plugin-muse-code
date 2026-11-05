# plugin-muse-code

Pohunek runtime plugin for [Meta's Muse Code](https://dev.meta.ai/docs/muse-code)
terminal coding agent.

| | |
|---|---|
| Repository | `pohunekd/plugin-muse-code` |
| Package id | `pohunek.runtime.muse` |
| Runtime id (`--agent`) | `muse` |
| Upstream | Meta Muse Code (proprietary CLI; the SDK and wire schema are MIT) |
| Status | **pre-release (P3)**: launch, program status, session identity, resume, subagents and transcript attribution (unverified end to end); no fork. The package is proven against Muse Code 1.4.14 (`compat/RELEASE`) and core `368a2d32c`, v0.39.0 (see `kit/Cargo.toml`) |

The plugin carries data and evidence only (descriptor, detect rules, captures).
It needs a core with every runtime feature its descriptor lists in `requires`
(`package/runtime.toml`); core 0.39.0 has all of them.
Handlers, reporters and parsers stay in the core repository
[`zajca/pohunek`](https://github.com/zajca/pohunek); the plan is the epic
`zajca/pohunek#1644` and the issues P0-P5 of this repository.

## What is here

| Path | Content |
|---|---|
| `package/runtime.toml`, `package/detect.toml` | The runtime package: descriptor and detection manifest, exactly what the archive holds |
| `kit/` | Stand-in for the plugin kit: a test crate that builds the package twice, installs it through core's definition path, reads the version banner and the process forms, and runs the detection manifest on every file of `compat/screens` and on the titles of `compat/osc7501` |
| `scripts/smoke` | End-to-end check against a pohunek build and an installed Muse: isolated daemon in a loopback-only namespace, launch, input, status, update |
| `scripts/capture` | Capture kit (bash, no third-party code): installs Muse into a throwaway HOME, drives it in a terminal and records what it shows |
| `scripts/hook-log` | Hook command that logs the stdin payload and the environment names of a hook call |
| `scripts/model-stub` | Loopback model endpoint (bash + jq) for approval and question screens without a model |
| `scripts/real-run` | Runner of the manual run with a real Meta model (#28): isolated daemon root, counting hooks, per-scenario step and wall caps, a spend stop line, aborts on an unexpected tool, evidence redaction and the scan that gates it |
| `compat/RELEASE` | The Muse release every capture comes from |
| `compat/install/` | `--version`, install layout, binary sha256, the process the launcher leaves running |
| `compat/help/` | `--help` of every command |
| `compat/screens/<scenario>/<width>.txt` | Terminal screens at widths 20 to 200: trust, login, api-key, idle, working, done, approval, question, update-notice, clear, fork, resume, seed |
| `compat/osc7501/` | OSC 7501 program-status records per scenario, terminal titles, the startup queries |
| `compat/hooks/<scenario>/` | One JSON file per hook call: event, stdin payload, environment names, the `MUSE_*`/`TBH_*` values |
| `compat/real/linux/` | Evidence of the real-model run per scenario: activity samples, screens, hook calls (long strings shortened), `ps` arguments, the runner's result; ids, paths and handshake values replaced by placeholders |
| `compat/macos-arm64/` | The same layout for macOS arm64: install, help, the login, api-key, trust, idle, working, done and update-notice screens, OSC 7501 records and titles, the default-HOME tree and the credential backend the sessions report |
| `compat/sessions/` | Session store tree and the first records of a session (strings over 80 characters become `<N chars>`; host fields such as pids and free disk space are dropped) |

## No credentials, no real model in automation

CI, the capture kit and `scripts/smoke` never log in, never store a credential
and never call a real model:

- Model-free turns use Muse's own `--provider echo`.
- Every step except `install` runs in a network namespace that has only a
  loopback interface, so no capture can reach Meta.
- Approval and question screens use `scripts/model-stub`, a loopback endpoint
  reached through `--base-url`. The CLI gets a placeholder `META_API_KEY` for
  that run only; the kit fails if any credential file appears.
- The browser sign-in entry of the login screen is never selected: Enter on it
  starts a device-code request at Meta.

A real Meta model runs only in the manual verification run of #28: through an
isolated Pohunek daemon root, in a throwaway repository with synthetic content,
within a budget the owner sets. It never runs in CI, in the capture kit or in
`scripts/smoke`. That run, too, never logs in to Meta through the browser
sign-in and never stores or prints a credential.

`scripts/real-run` drives that run. `setup` writes the agent profile without
a key (`profile` writes it again for a rerun after the profile was removed);
the operator adds `META_API_KEY` to its `[env]` and the runner never reads it, except inside `scan-profile`, which uses the value as the needle of
the evidence scan without printing it. The isolated daemon names an overlay
whose CLI the host lacks in its `overlays.toml`, so it serves its local socket
only. `scripts/real-run --help` lists the commands; a dry run against
`scripts/model-stub` (`setup --stub-port`) exercises every step without a
model.

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

On macOS the kit runs the screen and OSC steps (`install version login trust turn update-notice`; `hooks`, `sessions` and `stub` are refused) under a `sandbox-exec` profile, with bash 4+ and GNU sed, stat, find and date first on the PATH (for example from nix); `--out compat/macos-arm64` selects the evidence directory. The profile denies all socket traffic including loopback, allows unix sockets only below `--root` (tmux, at most 60 characters), denies the mach lookups that resolve names, reach the Security server or hand a URL to the user's browser, denies the real user's Keychain files, and denies `/usr/bin/open`. The browser hand-off (LaunchServices, `nsurlsessiond`) and `/usr/bin/open` denies are not probed. The credential backend is forced to a file for every launcher; the sessions report it in `compat/macos-arm64/sessions/credential-backend-*.txt`, and the kit fails if a session reports another backend. Before the first step the kit takes controls outside the sandbox (a name resolves, a loopback connection works, an item in a throwaway keychain under the root can be read) and requires each probe to fail inside it; the resolver socket is probed inside only. The sessions that ran under these denies report the file backend, which shows which backend was selected; it does not show that the CLI made no Security framework call. Its scan also looks for the host name, computer name, full name, serial number and hardware UUID, which it reads at run time.

The profile does not confine the filesystem: the CLI can still read the real user's home and write below its `~/Library`, so a dedicated throwaway user or a VM would be stronger. The default-HOME tree on macOS lacks the `runtime/` and `session-server` entries of the Linux run.

A new Muse release is captured by re-running the kit before the lock moves:
`supported.below` stays at the next minor until the new captures pass.

## Install and use

The plugin is not in the plugin index yet. Install it from a checkout:

```bash
cargo xtask package build /path/to/plugin-muse-code/package --output muse.tar.zst   # in zajca/pohunek
pohunek plugin install ./muse.tar.zst --sha256 sha256:<digest> --yes
pohunek plugin enable pohunek.runtime.muse
pohunek plugin select pohunek.runtime.muse
pohunek session new --agent muse
```

or, for development, `pohunek plugin link package --yes` followed by `enable` and
`select`. Muse itself is installed by its own installer
(`curl -fsSL https://dev.meta.ai/install.sh | sh`) and must be on the daemon's
`PATH`; sign in once (`muse`), or set an API key, before a session can do
work.

What the package does:

- Launches `muse --trust-workspace` with `MUSE_NO_AUTO_UPDATE=1` (owner decision
  2026-11-05: a Pohunek project is registered on purpose, so its workspace is
  trusted); a resume reuses the arguments its session was created with. A
  trusted workspace loads project hooks, rules and skills, and project hooks
  run outside Muse's sandbox. The flag trusts the run only; Muse saves no trust
  decision for it. See the checkout gap under Limits.
- Refuses `session input` with `session_input_blocked` while any dialog shows
  (trust, sign-in, API key, approval, question): Muse takes Enter as the choice
  of the preselected entry, so text sent while the trust dialog shows would
  trust the workspace unseen. Text sent while the approval dialog shows would
  allow a command unseen, and text sent while the question picker shows would
  pick an answer unseen; both are unverified.
- Accepts the release range `[1.4.14, 1.5.0)`, read from
  `Muse Code <release> (<release>-R<n>.<m>)`; the leading release is compared.
- Reports `idle`, `working` and `blocked` from the screen (composer, status
  line, trust, sign-in, API-key, approval and question dialogs), from the
  terminal title, and from OSC 7501 records (`app=muse`, the probe is answered).
- Reads Muse's session logs below `$XDG_DATA_HOME/muse/sessions` (else
  `~/.local/share/muse/sessions`), `YYYY/MM/DD/<uuid>/session.jsonl`, to attribute
  a Muse the user runs outside Pohunek; the kit checks the declared parser and
  root, and no run attributes such a Muse end to end (unverified). Recovery
  refuses a session whose `<uuid>/session.jsonl` is gone
  (`agent_native_reference_missing`).
- Types the first prompt once the composer is drawn (`ready = "screen_idle"`); the prompt is not an argument, so a prompt such as `logout` never selects a subcommand.

Limits:

- Trust follows the registered project, not the checked-out head: a
  registered repository whose checkout holds a foreign PR or fork head loads
  that head's `.muse` hooks, and they run outside Muse's sandbox with the
  operator's rights. Never run a Muse session on a foreign PR or fork checkout.
  Core issue [zajca/pohunek#1925](https://github.com/zajca/pohunek/issues/1925)
  lets the plugin pass the flag through `trusted_checkout_args` only for
  checkouts that pass a provenance check; #18 tracks the move.
- A host profile's `args` replace the descriptor's, so a profile over `muse`
  restates `--trust-workspace`. A profile that leaves it out shows the trust
  dialog in every root Muse does not trust yet; `session new --input` there
  fails with `session_input_not_ready` after the startup grace, and the session
  is rolled back: start it without `--input`, answer the dialog in
  `pohunek attach`, then send input. Muse keeps the answer per workspace root,
  so a new worktree asks again (architect probe, not a kit capture).
- Session identity comes from the root thread's `SessionStart` hook, which Muse
  reports with a turn (no capture shows one before the first turn): a session
  stopped before its first turn has no reference, and `session resume` refuses
  it (`native_identity_missing`).
- `/clear` does not move the reference on Muse 1.4.14: Muse runs the new
  session's hooks without the variables passed by `-c managed_hooks_env_vars`,
  so the reporter cannot reach its session, and `session resume` continues the
  conversation from before `/clear` (core issue pending).
- Fork is unsupported: Muse forks only inside its TUI (`/fork`).
- Background observer threads (`subagent_id` ending in `-reminder`) are never
  Pohunek subagents. A subagent the model spawns is reported with no role,
  because `SubagentStart` and `SubagentStop` carry none. They carry two ids:
  `subagent_id`, the spawn handle the model passes to `subagent_wait` and the
  subagent `id` in `session list`, and `child_session_id`, the child's session
  (equal to the hook's `session_id`; the child's own hooks run on that thread).
  The model's role shows only in the `PreToolUse` of `subagent_spawn`
  (`tool_input.role`) (real run, scenario 5).
- A Muse started by hand inside a Pohunek shell session is attributed through
  its process (`active_agent_base` `muse`), not through its session log;
  unverified end to end.
- No profile isolation (owner decision Q4): every profile shares Muse's config
  home, data root, trust list and stored sign-in, and profiles differ only by
  `args` and `[env]`. A profile that sets `META_API_KEY` in its `[env]` hands
  Muse that key; Pohunek never writes it into a descriptor, log or session
  record. `pohunek session new` cannot take the key from the caller's
  environment today.
- `Always allow in this workspace` is in the approval rule from prior art and
  is unverified. No rule names the approval profiles (`Ask me`, `Read-only`,
  `Unrestricted`); only `Auto-review` appears in a capture, in a footer no rule
  reads.

## Checks

```bash
(cd kit && cargo test --locked)          # descriptor, version, process, screens, titles
scripts/smoke --root /path/to/scratch \
  --core-bin /path/to/core/target/debug --muse-dir /path/to/muse/install   # the release in `compat/RELEASE`
```

`kit/` pins the core revision it is proven against (`kit/Cargo.toml`) and the
Rust toolchain of that revision (`kit/rust-toolchain.toml`). Move both together
with a new core release, after the nightly core has passed the same tests.

## Verified behaviour

Every row comes from a capture of the release in `compat/RELEASE` on Linux
x86_64 in a throwaway HOME. "Stub" means the behaviour was observed against the
loopback model stub, not a real model. Behaviour without a capture is listed as
unverified.

| Area | Result |
|---|---|
| Version line | `muse --version` prints `Muse Code 1.4.14 (1.4.14-R7309.1)`; the release id is `X.Y.Z-R<n>.<m>` |
| Install | Launcher script `muse` plus `.muse-version`, `.muse-release-info.json` and the static binary `muse-bin-<release>`; the pane process is the binary, so `comm` is `muse-bin-<release>` truncated by the kernel |
| Trust dialog | `Do you trust this workspace?` with `1  Trust and continue` / `2  Quit`; `--trust-workspace` skips it and writes no `trust.json` (`scripts/smoke`) |
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
| Package (P1) | `kit` tests: reproducible build with only `detect.toml` and `runtime.toml`, daemon accepts the descriptor, banner `Muse Code 1.4.14 (1.4.14-R7309.1)` is in range and 1.4.13, 1.5.0 and malformed banners are refused, the captured process matches, 55 screens of 15 scenarios read as the case table says, a spinner title reads working |
| Launch (P1, `scripts/smoke`) | Through a pohunek build of core `368a2d32c` (v0.39.0) in a loopback-only namespace: `--agent muse`, with `{"schema_version":1,"provider":"echo"}` in Muse's `settings.json`, launches `muse-bin-<release> --trust-workspace` with `MUSE_NO_AUTO_UPDATE=1` in its environment, shows no trust dialog, reaches the composer, renders an echo turn, and after stop `session resume` runs `muse-bin-<release> --trust-workspace -c managed_hooks_env_vars=[...] resume <id>`; no `trust.json` is left; with a profile of `--trust-workspace --provider echo` the initial input reaches the composer, `echo: <prompt>` renders, the session reads `working` during the delayed echo and `idle` after it, a second input works; the launcher records no update check and the release files stay unchanged |
| Identity and resume (P2, `scripts/smoke`) | With `--provider echo` the first turn binds the session to the Muse session id that names its `<uuid>/session.jsonl` directory, the reminder threads are filtered, and `session resume` runs `muse-bin-<release> <profile args, --trust-workspace among them> -c managed_hooks_env_vars=[...] resume <id>` in the workspace with `TBH_MANAGED_HOOKS_PATH` and `POHUNEK_SESSION_ID` in its environment and the reference unchanged |
| Recovery and profile key (P3, `scripts/smoke`) | With the session log moved away `session resume` is refused with `agent_native_reference_missing`; a profile with `--provider meta --base-url` on the loopback stub and a placeholder `META_API_KEY` in `[env]` runs a turn that renders the stub reply, and the placeholder appears in no file below `--root` except the profile |
| Input on the approval dialog (stub, `scripts/smoke`) | With the stub's scripted bash call the session reads `blocked` on `Would you like to run the following command?`, `session input` is refused with `session_input_blocked`, the command does not run and the dialog still shows |
| Update opt-out | In the launcher source `MUSE_NO_AUTO_UPDATE=1` skips the only code that downloads (`update_launcher`, `update_binary`) once a binary is installed; the smoke run saw no `.muse-update-checked-at` and no changed file |

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

### Real model, Linux (#28)

One manual run of `scripts/real-run` on 2026-11-05: Muse Code 1.4.14 with
`muse-spark-1.3-contributor`, `--reasoning-effort medium`, `--approval-judge
off`, contributor mode off (Meta accepted the model without
`--contributor-mode on`), through an isolated daemon of core 0.39.0, in a
throwaway repository holding one synthetic `README.md`. Evidence in
`compat/real/linux/<scenario>-<run>/`. Scenario 1 ran twice. Run 1 (`1-1`)
reads `fail` in its `result.json` because the runner revision of that run
(not committed) checked for no `PreToolUse` on any thread (check
`no-pretooluse`), and the observer threads called `submit_reminder_decision`;
the root-only count of the committed runner (`no-root-pretooluse`),
re-derived from the committed hooks, finds 0 root `PreToolUse` calls, and every other check of `1-1` passed.
Scenarios 6 and 7 continue the session of `1-1`. Run 2 (`1-2`) ran after the
check was limited to the root thread, in a new session that 6 and 7 never
touched. The same revision scored `2-1` and `3-1` by an all-thread count
(check `one-bash-pretooluse`), which passed only because the observer calls
came after the dialog; the root-only count re-derived from their hooks is one
root `PreToolUse` (`bash`) each. Dialogs were answered through `pohunek
attach`. 9 turns, 40 model calls (root, observers and the subagent). The spend
estimate from `PostLLMCall` usage (`spend.json`) is 1.20 USD, an upper bound:
all 830,963 input tokens priced as uncached (247,274 were cache reads) and
21,706 output plus 16,583 reasoning tokens as output, at the Muse Spark list
price. The `run_spend` inside each `result.json` was written during the run
and its `cached_tokens` counts the cache reads twice (Muse reports them in
two fields with the same value). The provider's own figure was not read.

| # | Scenario | Result |
|---|---|---|
| 1 | Plain turn | Pass (`1-2`; `1-1` reads `fail` only on the all-thread check above): activity `working` then `idle`, `◆ PONG`, `SessionStart` `source=startup`, the native reference is a UUIDv7 equal to the newest session directory; 1 root step, 16 s (`1-1`: 1 root step, 17 s). In both runs the root thread called no tool; the observer threads called `submit_reminder_decision` (`PreToolUse` on their own thread ids) |
| 2 | Approval, allow once | Pass: `blocked` on `Would you like to run the following command?` with the stage argv and `Allow this stage once (y)`; Enter through attach ran `echo approval-probe > approval-probe.txt` in the repository, `◆ DONE`, `idle`; 2 root steps, 31 s; one root `PreToolUse` (`bash`, workdir the repository); no `Notification` hook for the dialog |
| 3 | Approval, abort | Pass: the same dialog; Escape shows `denied` / `approval aborted`, no file is written, the model replies without retrying (one root `PreToolUse`), `idle` within the 60 s window after the Escape; 2 root steps, 48 s; no `Notification` hook |
| 4 | Question picker | Pass: `request_user_input` draws `Which colour?` with `red`, `blue` and Muse's own `None of the above`; `blocked` on `Enter to select`; Enter through attach answers `red`, `◆ red`, `idle`; 2 root steps, 42 s; the picker fires a root `Notification` (`notification_type` `question_prompt`) and no `PreToolUse` |
| 5 | Subagent | Pass: the model calls `subagent_spawn` (with `tool_input.role`) and `subagent_wait`; `SubagentStart` and `SubagentStop` carry no role, `subagent_id` (the spawn handle `subagent_wait` takes, also the subagent `id` in `session list`) and `child_session_id` (the child's session, equal to the hook's `session_id`); `session list` shows one subagent with `lifecycle` `completed`, `provider` `muse`, `parent_id` equal to the root's native reference; `◆ PARENT-DONE`; 3 root steps, 44 s |
| 6 | Stop and resume | Pass, on the session of `1-1`: after `session stop` and `session resume` the process runs `muse-bin-<release> <profile args> -c managed_hooks_env_vars=[...] resume <native id>`, `SessionStart` reports `source=resume`, the answer to the follow-up question is `◆ PONG`, the reference is unchanged; 1 root step, 21 s |
| 7 | `/clear` | Evidence, not a pass, on the session of `1-1`: `/clear` prints `started a fresh session` and `muse resume <old id>`; the new session's `SessionStart` (`source=clear`, a new id with its own session directory) carries no `POHUNEK_*` variable, so the reference stays on the session before `/clear` (F8, core issue C-B); `◆ FRESH`; 1 root step, 19 s |

Across the committed runs `state_source` was `osc_progress` in 89 samples, `process` in
3 and `screen` in 2. Under Pohunek the hook environment also held `LANG`,
`LOGNAME`, `OLDPWD`, `SHELL` and `TMPDIR` besides the names in the Hook
environment row. After the run the launcher had recorded no update check and
the install files were unchanged, with the network reachable.

### Unverified

- The first idle frame, the resume screens and the seed screen in a normal, non-namespaced run: unprivileged namespaces cannot map host root, so root-owned files appear owned by nobody and Muse prints the SSH-projection notice there. The footer itself is backed by `screens/done/<w>.txt` and `screens/working/<w>.txt` at every width.
- On macOS: hooks, session resume and fork, the stub screens, and the Keychain
  credential backend.
- Real approval variants beyond the single-stage shell command (`Always allow
  in this workspace`), `PostToolUseFailure`, compaction hooks, `StopFailure`,
  `Interrupt`.
- With a real model: transcript attribution of a Muse started outside Pohunek,
  and `session input` refused while the approval or question dialog shows (the
  real run answered every dialog through `pohunek attach`).
- A real-model run on macOS.
- When `SessionStart` is reported: the captures always include a turn, so reporting at launch versus with the first turn is not shown.
- That `Interrupt` and `ToolUseStart` hooks need `async: true` (a startup warning in an earlier exploration, not committed).
- The API-key prompt at width 20 (the wrapped login text defeats the kit's wait).
- Whether `TBH_MANAGED_HOOKS_PATH` overrides a `managed_hooks_path` set in
  `settings.json`.
- The terminal-side update banner driven by the release manifest (`urgency`,
  `notification_text`); the launcher's own update notice is captured.
- That `MUSE_NO_AUTO_UPDATE=1` leaves the network untouched for the binary itself (the launcher part is read from its source and observed; the namespace has no route, so an attempt would fail silently).
- The update-notice rule `launcher_update_notice` (the launcher line only; no TUI screen with a notice was captured).
- Any macOS behaviour of the package (P4).

Prior art cited by the plan (agent-deck, par-term, Zentty, Paradev, t3code,
openclaw) was not re-verified here.
