# Vetinari — OpenFang chief-of-staff agent

This is **Leo's fork of [RightNow-AI/openfang](https://github.com/RightNow-AI/openfang)**,
extended with everything needed to run it as `Vetinari` (a Lord-Vetinari-named chief-of-staff
agent) inside [Leo's homelab K3s cluster](https://github.com/l3ocifer/homelab).

The framework code itself lives at the repo root (it's a fork). All
homelab-specific additions live under `homelab/`. Upstream syncs via
`git fetch upstream && git merge upstream/main` (automated weekly by
`homelab/.github/workflows/upstream-sync.yml`).

## Layout

```
vetinari-openfang/                   ← repo root (this fork)
├── (upstream openfang source)
│   ├── crates/
│   ├── agents/
│   ├── hands/
│   ├── Cargo.toml
│   └── ...
└── homelab/                          ← everything we add
    ├── Dockerfile                    ← multi-stage: builds openfang from local source
    ├── k8s/                          ← kustomize tree (ArgoCD pulls this path)
    │   ├── kustomization.yaml
    │   ├── deployment.yaml
    │   ├── service.yaml
    │   ├── ingressroute.yaml
    │   ├── pvc.yaml
    │   └── rbac.yaml
    ├── config/                       ← Vetinari's persona + framework config
    │   ├── SOUL.md
    │   ├── TOOLS.md
    │   ├── openfang.toml
    │   └── agent.toml
    ├── shared/                       ← Git submodule → l3ocifer/homelab
    │                                   k8s/kustomization.yaml refs:
    │                                   ../shared/openclaw-configs/shared/{AGENTS,
    │                                   HANDOFF, KILLSWITCH, USER}.md
    ├── .github/workflows/
    │   ├── build.yml                 ← image build & push to GHCR
    │   └── upstream-sync.yml         ← weekly auto-PR `git pull upstream`
    ├── PATCHES.md                    ← what we changed vs upstream and why
    ├── CHANGELOG.md                  ← releases of vetinari-openfang
    └── README.md                     ← this file
```

## Deploying

ArgoCD's `vetinari` Application in
[`l3ocifer/homelab/argocd/apps/agents.yaml`](https://github.com/l3ocifer/homelab/blob/main/argocd/apps/agents.yaml)
points at this repo's `homelab/k8s` path. Push to `main` →
GitHub Actions builds + pushes
`ghcr.io/l3ocifer/vetinari-openfang:latest` → ArgoCD Image Updater
rolls the Deployment.

## Building locally

```bash
git clone --recurse-submodules git@github.com:l3ocifer/vetinari-openfang.git
cd vetinari-openfang
docker build -f homelab/Dockerfile \
  -t ghcr.io/l3ocifer/vetinari-openfang:latest \
  -t ghcr.io/l3ocifer/vetinari-openfang:$(git rev-parse --short HEAD) \
  .
```

For local dev with reduced build time:
`--build-arg LTO=false --build-arg CODEGEN_UNITS=16` drops a release
build from ~15 min to ~3 min at a small runtime perf cost.

## Syncing upstream manually

```bash
git fetch upstream main
git merge upstream/main         # may produce conflicts
# resolve conflicts in upstream files; touch PATCHES.md if a local
# patch needed re-applying
git push origin main
```

## Customizations vs upstream

See `homelab/PATCHES.md` for a curated list. The short version:

- `homelab/` directory (entirely additive — no upstream conflicts)
- The Dockerfile builds from local source instead of `git clone`,
  so we get reproducible images pinned to a specific commit
- Agent persona is `vetinari` with chief-of-staff-of-three-agents
  scope, defined in `homelab/config/{SOUL,TOOLS}.md` and
  `homelab/config/agent.toml`
- LiteLLM-routed inference (in-cluster gateway, not direct provider)

## Vetinari's persona, in 30 seconds

Calm. Observant. Concise to the point of being slightly chilling.
Diplomatic in a way that makes you suspect there is a great deal you
are not being told — because there is. Default-to-delegation. The
synthesizer, not the duplicator. Drafts not sends, always. Quiet
chief-of-staff is correct chief-of-staff. See `config/SOUL.md` for
the full picture.

## Required env vars

Provided by `vetinari-secrets` SealedSecret in the cluster (sealed
in `l3ocifer/homelab/argocd/sealed-secrets/vetinari-secrets.yaml.template`):

| Var | Use |
|---|---|
| `OPENFANG_API_KEY` | Bearer for HTTP API auth |
| `LITELLM_API_KEY` | In-cluster LiteLLM gateway |
| `ANTHROPIC_API_KEY` | Cloud fallback when LiteLLM is degraded |
| `DATABASE_URL` | `postgres://openfang_vetinari@homelab-pg-rw...` |
| `VETINARI_RO_PASSWORD` | psql for sibling agent DBs (read-only) |
| `MATRIX_HOMESERVER` + `MATRIX_ACCESS_TOKEN` | `@vetinari:leopaska.xyz` |
| `TELEGRAM_BOT_TOKEN` | shared homelab Telegram bot |
| `NTFY_TOKEN` | bearer for ntfy.leopaska.xyz |
| `BLUEBUBBLES_API_KEY` | shared key with bluebubbles-proxy |
| `OFP_SHARED_SECRET` | HMAC-SHA256 mutual auth for A2A |
| `OP_SERVICE_ACCOUNT_TOKEN` | 1Password service-account |

## License

OpenFang upstream: MIT/Apache-2.0 (see [LICENSE](../LICENSE)).
Homelab additions in `homelab/`: same. Persona text in
`homelab/config/SOUL.md` is intellectual property of Leo Paska.
