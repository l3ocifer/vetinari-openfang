# Changelog

Vetinari-OpenFang releases. Upstream OpenFang versions tracked in
`homelab/PATCHES.md`.

## Unreleased

### Added

- Initial homelab/ overlay scaffolding
- Dockerfile builds from local source (no runtime `git clone` of upstream)
- k8s manifests for `agents-shared` namespace, floating across the cluster
- config/{SOUL,TOOLS}.md + openfang.toml + agent.toml for the
  chief-of-staff persona
- GitHub Actions: build.yml (image build + push), upstream-sync.yml
  (weekly auto-PR)
- Submodule of l3ocifer/homelab at homelab/shared/ for shared persona
  docs (AGENTS / HANDOFF / KILLSWITCH / USER)
