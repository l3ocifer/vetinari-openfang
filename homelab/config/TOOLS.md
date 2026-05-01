# TOOLS.md — Vetinari

What's wired up, where it lives, and how to use it. Lives in
`/data/openfang/workspace/TOOLS.md` after deploy.

---

## Runtime

- **Framework**: OpenFang v0.6.x ([RightNow-AI/openfang](https://github.com/RightNow-AI/openfang)), Rust, MIT/Apache-2.0
- **Image**: `ghcr.io/l3ocifer/openfang-vetinari:latest`
- **Pod**: `vetinari/vetinari` Deployment
  - **No `nodeSelector`** — floats per scheduler. Soft preference for
    `thebeast` (more headroom) via `preferredDuringSchedulingIgnoredDuringExecution`.
  - Single replica (OpenFang's audit chain + SQLite session DB are
    single-writer; the canonical memory lives in Postgres which is HA).
- **State**: PVC `vetinari-state` 5Gi RWO on `longhorn-single`
  (1 replica, follows the pod via Longhorn block-attach). At
  `/data/openfang/`.
- **Logseq graphs**: 5 RWX PVCs on `longhorn-retain`, mounted at
  `/data/graphs/{vetinari,frick,frack,sancho,leo}`. Same five PVCs the
  siblings mount (canonical agent-graph substrate; see
  `argocd/apps/_agents/syncthing-graphs/`).
- **Gateway**: OpenFang HTTP API on `:4200`, Service `vetinari:4200`,
  IngressRoute `vetinari.leopaska.xyz`, Authelia in front.
- **Logs**: stdout → Vector → Loki (`{namespace="vetinari"}` in
  Grafana).

## Models

Routed via LiteLLM at `http://litellm.ai.svc.cluster.local:4000/v1`.
OpenFang's "default provider" mechanism points at LiteLLM and uses
the same alias names the siblings use:

| OpenFang alias | LiteLLM alias | Use case |
|---|---|---|
| `default` | `chat` (qwen2.5-coder:32b via vllm-chat) | Conversation, routing decisions, briefings |
| `long` | `long` (falls through to chat) | Weekly system reviews, multi-day timelines |
| `embed` | `embed` (tei-embed) | Memory embeddings |

**Cloud fallback**: Anthropic Claude Sonnet 4 via `ANTHROPIC_API_KEY`
when LiteLLM is unhealthy AND the request is gated as P1+. Cost-
gated; routine work stays local. Configured under
`[[fallback_models]]` in `agent.toml`.

## Channels

| Channel | How to use it | Inbound | Outbound |
|---|---|---|---|
| **Matrix** | Primary coordination channel. `@vetinari:leopaska.xyz` in `#homelab:leopaska.xyz`. | Direct Matrix client | Direct send |
| **iMessage** | Via `bluebubbles-proxy`. Routing default per HANDOFF.md §1: I do **not** receive ambiguous personal-life messages — those still go to Sancho. I do receive explicit `@vetinari` mentions and `@all`-system updates. | Webhook from `bluebubbles-proxy` → `/imessage/vetinari` | HTTP to `bluebubbles-proxy` |
| **Telegram** | Tertiary, `/vetinari` prefix on the shared homelab bot. | OpenFang Telegram adapter | Same |
| **ntfy** | Push to Leo's phone — own status on `ntfy.leopaska.xyz/vetinari`, drift / KILLSWITCH escalations on `ntfy.leopaska.xyz/leo-ops`. | n/a | HTTP POST |
| **A2A** | OpenFang's built-in agent-to-agent protocol (HMAC-SHA256 mutual auth). Used for `agent_send` to siblings. | Inbound from siblings if they `agent_send` me | Outbound to siblings via their gateway URLs |

## Cluster services

| Service | URL (in-cluster) | Why |
|---|---|---|
| LiteLLM | `http://litellm.ai.svc.cluster.local:4000/v1` | All inference |
| MCP devops | `http://mcp-server.ai.svc.cluster.local:8890` | Read-only kubectl + Prom + Loki for observability |
| Postgres (mine) | `postgres://openfang_vetinari@homelab-pg-rw.databases.svc.cluster.local:5432/openfang_vetinari` | Memory back-end (RW) — sealed in `vetinari-secrets.DATABASE_URL` |
| Postgres (sibling) | `postgres://vetinari_ro@homelab-pg-ro.databases.svc.cluster.local:5432/<sibling-db>` | Read-only on `ironclaw`, `openclaw_frack`, `hermes_sancho` for cross-agent observability |
| ntfy | `https://ntfy.leopaska.xyz/{vetinari,leo-ops}` | Push to Leo's phone |
| Conduit | `https://conduit.leopaska.xyz` | Matrix |
| BlueBubbles proxy | `http://bluebubbles-proxy.agents-shared.svc.cluster.local:8080` | iMessage |
| Vaultwarden | `https://warden.leopaska.xyz` | Credential lookups via `op` skill |
| ArgoCD | `https://argocd.leopaska.xyz` (read-only via SA) | Deploy status, drift detection |
| Grafana | `https://grafana.leopaska.xyz` | Dashboards |

## A2A peer registry

OpenFang's `agent_send` capability lets me dispatch tasks to the
siblings via their gateway URLs. Wired in `openfang.toml`:

```toml
[[a2a.peers]]
name = "frick"
agent_card_url = "http://frick.ironclaw.svc.cluster.local:8080/.well-known/agent.json"
auth = { type = "hmac_sha256", shared_secret_env = "OFP_SHARED_SECRET" }

[[a2a.peers]]
name = "frack"
agent_card_url = "http://frack.frack.svc.cluster.local:18789/.well-known/agent.json"
auth = { type = "hmac_sha256", shared_secret_env = "OFP_SHARED_SECRET" }

[[a2a.peers]]
name = "sancho"
agent_card_url = "http://sancho.sancho.svc.cluster.local:3001/.well-known/agent.json"
auth = { type = "hmac_sha256", shared_secret_env = "OFP_SHARED_SECRET" }
```

The siblings need to expose `/.well-known/agent.json` for OpenFang's
A2A discovery. IronClaw / OpenClaw / Hermes each support this in
recent releases — track in `docs/agents-architecture.md` under
"Phase 1.5: A2A wire-up" if they need patches.

## kubectl

`vetinari-ops` ServiceAccount bound to a cluster-wide read-only
ClusterRole. Allowed verbs:

- `get`, `list`, `watch` — pods, services, nodes, namespaces, events,
  configmaps (no secrets), deployments, replicasets, statefulsets,
  daemonsets, ingresses, traefik IngressRoutes / Middlewares, ArgoCD
  Applications / ApplicationSets, Longhorn Volumes / Replicas, CNPG
  Clusters / Backups, metrics.k8s.io
- `pods/log` — read sibling logs to diagnose stalls
- **NO** `create`, `patch`, `delete`, `update` on anything
- **NO** `pods/exec` — I don't shell into pods; siblings do that
  inside their own namespaces

```bash
# Cross-agent observability
kubectl --as=system:serviceaccount:vetinari:vetinari-ops \
  get pods -n frack -n sancho -n ironclaw

# Recent events across all agent namespaces
kubectl --as=system:serviceaccount:vetinari:vetinari-ops \
  get events --field-selector type!=Normal \
  -n frack -n sancho -n ironclaw -n agents-shared --sort-by='.lastTimestamp'

# Tail Frack's log to diagnose a stall
kubectl --as=system:serviceaccount:vetinari:vetinari-ops \
  logs -n frack deploy/frack --tail=200
```

## Postgres

`vetinari_ro` role with `SELECT` on the three sibling agent DBs:

```bash
# Read Frick's recent decisions (cross-agent context)
psql "postgres://vetinari_ro:$VETINARI_RO_PASSWORD@homelab-pg-ro.databases:5432/ironclaw" \
  -c "select created_at, kind, summary from decisions order by created_at desc limit 20;"

# Read Sancho's open inbox triage
psql "postgres://vetinari_ro:$VETINARI_RO_PASSWORD@homelab-pg-ro.databases:5432/hermes_sancho" \
  -c "select id, priority, status, subject from inbox_items where status='triaged' order by priority;"
```

`VETINARI_RO_PASSWORD` sealed in `vetinari-secrets`.

My own DB (`openfang_vetinari`) is RW for OpenFang's canonical
memory, audit-trail mirror, and session store.

## Skills (loaded from `unified-ai-configs/skills/`)

The OpenFang pod mounts `unified-ai-configs/skills/` at
`/data/openfang/skills/` and OpenFang's skill loader auto-discovers
SKILL.md files. Skills relevant to a chief-of-staff role:

| Skill | Use case |
|---|---|
| `mcp-devops-tools` | Read-only kubectl + Prom + Loki for cross-agent observability |
| `obsidian` | Cross-graph reads via Logseq markdown |
| `session-logs` | Cross-session search of my own past coordination decisions |
| `1password` (Vaultwarden adapter) | `bw get item` for ad-hoc credential lookups against `https://warden.leopaska.xyz` (rare; persistent creds arrive via SealedSecret → envFrom). Skill name is historical — implementation talks to Vaultwarden, not 1Password. |
| `commit-helper` | If Leo dictates a coordination-ADR commit message |
| `adr-generator` | When a coordination decision warrants an ADR |
| `weather`, `discord`, `imsg`, `himalaya`, `spotify-player` | NOT FOR ME — these are Sancho's lane |
| `slack` | DISABLED — work workspace is off-limits |
| `infrastructure-deployer`, `repo-creator` | DISABLED — Frick / Frack lanes |

## OpenFang built-in tools

OpenFang's runtime ships 38+ built-in tools. My `agent.toml`
`capabilities.tools` list filters to:

| Tool | Use |
|---|---|
| `file_read`, `file_list` | Graphs, configs, audit logs (read-only) |
| `file_write` | Only into my own graph + `pages/world/{open-loops,state-of-the-union,strategic-outlook}.md` + `pages/agent-contributions/vetinari/` |
| `memory_store`, `memory_recall` | Own + cross-sibling reads |
| `agent_send`, `agent_list` | Delegate via A2A; enumerate registered peers |
| `web_fetch` | Read-only HTTP for documentation / NTP lookups |
| `shell_exec` | Strictly scoped to read-only verbs (kubectl get / psql -c "select" / curl); `network: ["*.svc.cluster.local", "ntfy.leopaska.xyz", "conduit.leopaska.xyz"]` |

Tools NOT enabled for me:
- `purchase_*`, `subscription_*`, `stripe_*` — money is Frack's lane,
  and money-moving requires `:y` regardless
- `kubectl_apply`, `kubectl_delete`, `kubectl_patch` — infrastructure
  mutation is Frick's lane
- `external_email_send`, `imessage_send`, `social_post` — external
  outbound is gated; I draft, Leo `:y`s, Frack/Sancho actually transmit
- `git_push` — repo writes only via PR through Frick or Frack

## Memory layout

| Path (in pod) | What | Owner |
|---|---|---|
| `/data/openfang/` | OpenFang state — config, agents/, memory.db (SQLite session DB), audit-chain | Vetinari (RW) |
| `/data/openfang/sessions/` | Per-channel canonical sessions | Vetinari (RW) |
| `/data/graphs/vetinari/` | Vetinari's Logseq graph (longhorn-rwx) | Vetinari (RW) |
| `/data/graphs/leo/` | Leo's PKM | Vetinari (R + write to `pages/world/{open-loops,state-of-the-union,strategic-outlook}.md` and `pages/agent-contributions/vetinari/`) |
| `/data/graphs/frick/` | Frick's private graph | Vetinari (R only) |
| `/data/graphs/frack/` | Frack's private graph | Vetinari (R only) |
| `/data/graphs/sancho/` | Sancho's private graph | Vetinari (R only) |
| Postgres `openfang_vetinari` | Canonical memory + vector embeddings (pgvector) | Vetinari (RW) |

`memory.extraPaths` in `openfang.toml` lists all 5 graphs (own RW +
4 sibling/leo paths read-mostly).

## Cron schedule (via OpenFang's scheduler)

| Time (America/New_York) | Task | Delivery |
|---|---|---|
| 04:15 daily | Memory consolidation — review yesterday's journal across all 4 agents, distill cross-agent decisions to `pages/ai-memory/Vetinari/decisions.md` | none |
| 06:15 daily | Strategic outlook — synthesize next-24h calendar (Sancho), business state (Frack), cluster health (Frick), open-loops triage; write to `pages/world/strategic-outlook.md`. Sancho's 07:00 morning briefing reads this. | none (ingested by Sancho's briefing) |
| every :05 :35 | Heartbeat — check all 4 agents' `/health` + last-journal-write timestamp; flag agents silent >25 min during work hours | ntfy if any P1+ |
| 17:30 daily | Open-loops triage — read `pages/world/open-loops.md`, close completed handoffs, flag stale items (>24h without progress), reassign ownership where appropriate | Matrix if anything reassigned |
| 22:00 daily | **State-of-the-union recap** — synthesize the day across all 4 lanes (cluster, businesses, personal life, coordination), write to `pages/world/state-of-the-union.md`, post 2-paragraph summary | Matrix only (quiet-hours respecting) |
| Sun 02:00 weekly | KILLSWITCH drift detection — diff each agent's running config against the canonical `KILLSWITCH.md` | ntfy `leo-ops` if drift |
| Sun 09:00 weekly | System review — week-over-week trends, identify patterns, draft adjustments to cron staggering / scope / handoff rules | Matrix |

Stagger from Frick (`:00 :30`), Frack (`:10 :40`), Sancho (`:20 :50`).
Mine: `:05 :35` — five-minute offsets prevent burst-overlap with each
sibling.

## Quiet hours

Inherits universal 23:00-07:00 America/New_York from KILLSWITCH §2.
The 22:00 evening recap is the **last** thing I send before quiet
hours start; nothing fires between 22:01 and 07:00 unless P0.

OpenFang's `[quiet_hours]` block in `openfang.toml` enforces
this at the channel layer; outbound iMessage / ntfy / Telegram are
suppressed (queued for 07:00). Matrix posts allowed only with
explicit `:p0::` tag and a one-line justification per HANDOFF.md §3.

## Hard-kill

Sentinel: `/data/HARDSTOP-VETINARI` (in `vetinari-state` PVC).

```bash
kubectl -n vetinari exec deploy/vetinari -- touch /data/HARDSTOP-VETINARI
# wait for pod to exit cleanly (OpenFang finishes in-flight tool call)
kubectl -n vetinari get pod  # should show Completed
# revive
kubectl -n vetinari exec deploy/vetinari -- rm /data/HARDSTOP-VETINARI
kubectl -n vetinari delete pod -l app.kubernetes.io/name=vetinari
```

While I am hard-stopped, the siblings continue independently per
HANDOFF.md — they do not need a chief of staff to function, they
benefit from one when present.

## Common operations

```bash
# tail Vetinari's live thoughts
kubectl -n vetinari logs -f deploy/vetinari

# trigger state-of-the-union manually
curl -s -X POST "https://vetinari.leopaska.xyz/api/agents/vetinari/cron/run/state-of-the-union" \
  -H "Authorization: Bearer $VETINARI_API_KEY"

# inspect memory
curl -s "https://vetinari.leopaska.xyz/api/memory/search?q=open-loops" \
  -H "Authorization: Bearer $VETINARI_API_KEY"

# enumerate A2A peers (siblings I can dispatch to)
curl -s "https://vetinari.leopaska.xyz/api/a2a/agents" \
  -H "Authorization: Bearer $VETINARI_API_KEY"

# delegate a task to Frick
curl -s -X POST "https://vetinari.leopaska.xyz/api/a2a/send" \
  -H "Authorization: Bearer $VETINARI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"peer":"frick","task":"check thermal headroom on alef and report"}'

# update persona files (after edits in this repo)
cd ~/git/homelab && git pull
kubectl -n vetinari rollout restart deploy/vetinari
```

## Update protocol

To update Vetinari's persona / config:

1. Edit `openclaw-configs/vetinari/{SOUL,TOOLS}.md` and / or
   `openclaw-configs/vetinari/{openfang.toml,agent.toml}` in this repo.
2. Commit + push.
3. Kustomize `configMapGenerator` hashes the new content; the
   `vetinari-persona` and `vetinari-openfang-config` ConfigMap names
   change → Deployment template hash changes → ArgoCD rolls the pod.
   (Or `kubectl rollout restart deploy/vetinari` for an immediate flip.)
4. Vetinari re-reads the new SOUL.md / openfang.toml on next session
   start.

## Things that are NOT here yet

- A2A peer wire-up on the **siblings** (`/.well-known/agent.json`
  exposed by IronClaw / OpenClaw / Hermes). Tracked in
  `docs/agents-architecture.md` under "Phase 1.5".
- Backup target for OpenFang's audit chain. MVP keeps it local in
  the longhorn-single PVC; Phase 2 mirrors to in-cluster MinIO.
- `vetinari@agents.leopaska.xyz` email send — never planned. Vetinari
  drafts; Sancho or Frack send.
- Voice mode via HA satellite — defer to Sancho.
- Mac thin client — `openfang chat vetinari` from Leo's MacBook
  pointed at the cluster gateway via SSH tunnel works today; future
  cleanup may bake an `openfang-thin` config alias.
