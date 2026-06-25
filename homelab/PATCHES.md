# Local patches vs upstream RightNow-AI/openfang

Track non-additive changes (i.e. anything that touches files outside
`homelab/`). Additive changes — files added under `homelab/` —
do not need entries here, since they cannot conflict with upstream.

## Active patches

### runtime-autonomous-graph-write: let autonomous turns persist to graph + run gated tools

- **Files**: `crates/openfang-kernel/src/kernel.rs`, `crates/openfang-runtime/src/{agent_loop,tool_runner,workspace_sandbox,apply_patch}.rs`, `crates/openfang-types/src/agent.rs`, `crates/openfang-api/src/routes.rs`
- **Reason**: deployed `builtin:chat` autonomous turns could not write to `/data/graphs/<self>/pages` (file_write sandboxed to the workspace) and could not run gated tools unattended (no approver on a cron turn). Adds `[capabilities].file_roots` to extend the write sandbox and `[approval].auto_approve_autonomous` to auto-approve tool calls on turns whose manifest has `[autonomous]`. This is what lets the mission-orchestrator's file-existence DONE gate ever pass for Vetinari.
- **Config that activates it**: `homelab/config/agent.toml` (`file_roots = ["/data/graphs"]`, `[autonomous]`), `homelab/config/openfang.toml` (`[approval].auto_approve_autonomous = true`).
- **Note**: the graph `pages/world` dir must be writable by uid 1000 — the deterministic root CronJobs create it root-owned, so `homelab/k8s/deployment.yaml` adds a `seed-graph-world` init container that chowns it. Without that, file_write still fails with EACCES even with this patch.
- **Upstream PR**: not yet submitted (homelab-specific capability semantics).
- **Last applied**: 2026-06-25 (commit 2d4ddfa), openfang 0.6.10.

## Resolving an upstream merge conflict

When `git merge upstream/main` reports a conflict in a file we patch:

1. Identify the patch from the list below
2. Re-apply it to the merged result
3. Bump `Last applied` for that patch
4. Commit with message `merge: upstream <sha> + reapply <patch-id>`

## Format for new patches

```markdown
### <patch-id>: <short description>

- **File**: path/to/file.rs
- **Reason**: why we changed it
- **Upstream PR**: link if we sent a PR to upstream
- **Last applied**: 2026-04-30 against upstream@abc1234

\`\`\`diff
- ...
+ ...
\`\`\`
```
