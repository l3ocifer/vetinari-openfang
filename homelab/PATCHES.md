# Local patches vs upstream RightNow-AI/openfang

Track non-additive changes (i.e. anything that touches files outside
`homelab/`). Additive changes — files added under `homelab/` —
do not need entries here, since they cannot conflict with upstream.

## Active patches

_(none today — Vetinari ships pure-vanilla OpenFang, customized only
via configs in `homelab/config/`)_

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
