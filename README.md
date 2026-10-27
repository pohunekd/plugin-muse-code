# plugin-muse-code

Pohunek runtime plugin for [Meta's Muse Code](https://dev.meta.ai/docs/muse-code)
terminal coding agent.

| | |
|---|---|
| Repository | `pohunekd/plugin-muse-code` |
| Package id | `pohunek.runtime.muse` |
| Runtime id (`--agent`) | `muse` |
| Upstream | Meta Muse Code (proprietary CLI; the SDK and wire schema are MIT) |
| Status | **pre-release**: no descriptor yet, evidence only; nothing is verified yet |

The plugin carries data and evidence only (descriptor, detect rules, captures).
Handlers, reporters and parsers stay in the core repository
[`zajca/pohunek`](https://github.com/zajca/pohunek); the plan is the epic
`zajca/pohunek#1644` and the issues P0-P5 of this repository.

## No credentials, no real model

There is no Meta API key for this project and none will be created.
The capture kit (added in the next change) never logs in, never stores a credential and never calls a real model:

- Model-free turns use Muse's own `--provider echo`.
- Approval and question screens use `scripts/model-stub`, a loopback endpoint
  reached through `--base-url`. The step runs in a network namespace that has
  only a loopback interface, so nothing can reach a real endpoint. The CLI gets a
  placeholder `META_API_KEY` for that run only; no credential file is written.
- The browser sign-in entry of the login screen is never selected: Enter on it
  starts a device-code request at Meta.

