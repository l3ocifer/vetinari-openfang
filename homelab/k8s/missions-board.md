title:: missions
type:: mission-board
icon:: 🎯
owner:: vetinari
seeded:: 2026-06-01

- # 🎯 Fleet Mission Board
  Standing per-agent missions. **Vetinari owns this page.** `kanban-tick` (hourly) dispatches `TODO`→`DOING` via the agent-bus and reconciles `DOING`→`DONE` from `fleet_events` + deliverables. `open-loops-scan` (30m) chases stalled `DOING` blocks. Leo edits priority/scope/acceptance freely; Vetinari never deletes a block.
  Block lifecycle: `TODO` (not yet dispatched) → `DOING` (handed off, in flight) → `DONE` (deliverable verified). `task-id` is the stable A2A correlation key — never change it.
- ## Missions
- TODO Puck — self-hosted text-to-video pipeline
  assigned:: puck
  task-id:: mission-puck-t2v
  priority:: P1
  deliverable:: /data/graphs/puck/pages/world/t2v-pipeline.md
  acceptance:: a documented, reproducible text→video workflow (ComfyUI or equiv) scaled >0, plus one rendered sample committed and referenced from the deliverable page
  notes:: ComfyUI parked at 0 replicas as of 2026-06-01; un-parking is gated on internal inference being solid — coordinate with Quirm/Vetinari before scaling GPU.
- TODO Puck — author-style compendium model (Unsloth fine-tune) for AuthorWorks
  assigned:: puck
  task-id:: mission-puck-author-model
  priority:: P1
  deliverable:: /data/graphs/puck/pages/world/author-model.md
  acceptance:: fine-tuned style-conditioned model (Pratchett, Gaiman, Adams, Ringo, Rothfuss, Frankowski, Zelazny, Vinge, Butcher, Banks, Stephenson compendium — roster is modular, see corpus registry) trained via Unsloth QLoRA, evals passed (fine-tune preferred ≥70% on style-conditioned pairs), served behind LiteLLM alias `author`, verified against AuthorWorks generation
  notes:: full charter in Forgejo issue puck-openclaw#27; infra support (GPU burst lane + `author` alias) is homelab#57 routed to Frick. Corpus lives on blade at /srv/archive/corpus (authors.toml registry + corpus.py scan/collect; source in homelab scripts/corpus/) — Pratchett found+staged, 10 authors pending acquisition. Vimes rights-review checkpoint blocks training. Vimes lock, if applied, is final until cleared.
- TODO Quirm — hardware inference optimization + synthesis
  assigned:: quirm
  task-id:: mission-quirm-infer
  priority:: P0
  deliverable:: /data/graphs/quirm/pages/world/inference-optimization.md
  acceptance:: bench.findings synthesized into a ranked model×hardware recommendation table (tokens/s, VRAM, quality tier per node), handed to Vetinari for scheduling via handoff.completed
  notes:: bench loop is healthy (starvation false-alarm fixed 2026-06-01). Gap is synthesis→recommendation, not data collection.
- TODO Vetinari — inference scheduling optimization
  assigned:: vetinari
  task-id:: mission-vet-sched
  priority:: P1
  deliverable:: /data/graphs/vetinari/pages/world/scheduling-policy.md
  acceptance:: schedule placement consumes Quirm's model×hw table; documented policy + measured before/after (queue latency, GPU utilization)
  notes:: scheduling API + timeout fixes shipped 06-01 (Leo-driven). This mission is the data-driven optimization layer on top, blocked on mission-quirm-infer.
- TODO Vimes — security posture tracking
  assigned:: vimes
  task-id:: mission-vimes-sec
  priority:: P1
  deliverable:: /data/graphs/vimes/pages/world/threats.md
  acceptance:: continuously maintained threat + audit log with CVE tracking, surfaced to fleet_events as security.* events; refreshed at least daily
  notes:: today only infra-level falco/crowdsec exists; the agent-owned tracking layer is the mission.
- TODO Frick — infrastructure tracking
  assigned:: frick
  task-id:: mission-frick-infra
  priority:: P1
  deliverable:: /data/graphs/frick/pages/world/cluster-state.md
  acceptance:: maintained cluster-state.md (capacity trend, restart/OOM patterns by namespace, GitOps drift) refreshed daily and consumed by Vetinari's cluster-deep-dive
  notes:: infra itself is healthy via GitOps; the mission is the agent-driven observation/tracking cadence.
- TODO Frack — production apps to v1
  assigned:: frack
  task-id:: mission-frack-v1
  priority:: P1
  deliverable:: /data/graphs/frack/pages/world/prod-apps-v1.md
  acceptance:: inventory of all production apps with a per-app v1 readiness checklist (auth, backups, monitoring, docs, custom domain) + explicit gap list
  notes:: apps are healthy via ArgoCD but not frack-driven; the mission is owning the v1 readiness bar across the 12-business surface.
