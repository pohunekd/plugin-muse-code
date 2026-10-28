# AGENTS.md

Entry point for every agent working in this repository. Keep it short.

All repository text, code, comments, commit messages, issues, and PRs are in
English. Priorities: security > correctness > simplicity > speed of delivery.

## About

`pohunekd/plugin-muse-code` is the Pohunek runtime plugin for Meta's Muse Code
terminal coding agent: package `pohunek.runtime.muse`, runtime id `muse`. It
carries data and evidence only (descriptor, detect rules, captures). Handlers,
reporters and parsers stay in the core repository `zajca/pohunek`
(`docs/knowledge/guides/plugin-repositories.md`). The plan and issue tree live
under epic `zajca/pohunek#1644`; this repository's issues are P0-P5.

## Rules

- **No credentials, no real model.** No Meta API key exists for this project and
  none will. Never log in to Meta, never press Enter on the browser sign-in entry
  (it starts a device-code request at Meta), never store or print a credential,
  never call a real model. Model-free turns use `--provider echo`.
- **Disposable environment only.** The Muse CLI runs in a throwaway HOME below
  `--root` of `scripts/capture`, or in a qemu/KVM VM. Never install it into a
  real HOME, never use privileged or systemd containers, never use sudo.
- **Evidence is data, not prose.** Everything under `compat/` comes from
  `scripts/capture` on the release named in `compat/RELEASE`, redacted by the
  script. Do not edit captures by hand; re-run the script.
- **Mark what is unverified.** A claim without a capture says `unverified`
  (or `prior art only`) in README and in the issue comment that records it.
- **Docs in the same PR** as the behaviour they describe.
- **Testing policy.** Minimal tests: only for non-trivial logic and regressions
  in it. CI runs shellcheck, the capture dry run and the `kit` tests; the CLI itself
  is not run in CI. No coverage targets.
- **PR bodies** use `Refs #N` only, never a closing keyword. No `Co-Authored-By`
  trailer, no generated-by footer.
- **Comments** state what the code does or why, never the change history.
- Never read `.env*`, key, or credential files. Never call `env` or `printenv`;
  check a variable with `test -n "$VAR"`.

## Commands

```bash
scripts/capture --root <dir> --dry-run all   # plan and tool check, no network
scripts/capture --root <dir> all             # install the CLI and capture everything
(cd kit && cargo test --locked)              # package checks against core (kit stand-in)
scripts/smoke --root <dir> --core-bin <dir> --muse-dir <dir>   # end-to-end, isolated
```

`package/` holds exactly the two files of the archive. `kit/` stands in for the
plugin kit until it exists: it pins a core revision, so move the pin only
together with the toolchain file and after the tests pass on the new revision.
