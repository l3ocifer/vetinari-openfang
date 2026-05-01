# vetinari-openfang/homelab/Dockerfile
#
# Builds the OpenFang Agent OS image for **Vetinari**, the chief-of-
# staff agent for Leo's homelab. The build context is the repository
# root (this fork of RightNow-AI/openfang) — the upstream source is
# right there, plus any patches we have layered on top.
#
# This Dockerfile mirrors upstream's Dockerfile structure (so upstream
# build changes are easy to merge in) but adds the runtime tools
# Vetinari needs in-cluster (kubectl, op, postgresql-client, jq, rg)
# and the workspace skeleton at /data/openfang/.
#
# Build (from repo root):
#   docker build -f homelab/Dockerfile \
#     -t ghcr.io/l3ocifer/vetinari-openfang:latest \
#     -t ghcr.io/l3ocifer/vetinari-openfang:$(git rev-parse --short HEAD) .
#
# CI/CD: GitHub Actions (homelab/.github/workflows/build.yml) runs
# this build on every push to main and tags. ArgoCD Image Updater
# (newest-build strategy) rolls Vetinari's Deployment automatically.

# syntax=docker/dockerfile:1

# ─── Stage 1: builder ────────────────────────────────────────────────
# Mirrors upstream Dockerfile lines 1-16 — keep this list aligned with
# upstream so merges are clean. Adding apt deps for openssl/etc which
# upstream already requires.
FROM rust:1-slim-bookworm AS builder
WORKDIR /build
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      pkg-config libssl-dev perl make ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Workspace members per Cargo.toml:
#   crates/* + xtask. Plus agents/ (runtime templates) and packages/
#   (potentially used by build scripts) per upstream Dockerfile.
COPY Cargo.toml Cargo.lock ./
COPY crates   ./crates
COPY xtask    ./xtask
COPY agents   ./agents
COPY packages ./packages
# rust-toolchain pin — upstream uses it, we honor it
COPY rust-toolchain.toml ./

ARG LTO=true
ARG CODEGEN_UNITS=1
ENV CARGO_PROFILE_RELEASE_LTO=${LTO} \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS=${CODEGEN_UNITS}

RUN cargo build --release --bin openfang \
 && strip target/release/openfang

# ─── Stage 2: runtime ────────────────────────────────────────────────
# Upstream uses rust:1-slim-bookworm as runtime (huge — keeps cargo +
# all build deps in the image). For our cluster image we shrink to
# debian-slim + only the runtime tools Vetinari actually invokes.
FROM debian:bookworm-slim

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      ca-certificates curl wget tini \
      python3 python3-pip python3-venv \
      jq ripgrep git unzip \
      postgresql-client \
      gnupg lsb-release \
 && rm -rf /var/lib/apt/lists/*

# kubectl matched to current K3s cluster
ARG KUBECTL_VERSION=v1.34.0
RUN curl -fsSL "https://dl.k8s.io/release/${KUBECTL_VERSION}/bin/linux/amd64/kubectl" \
      -o /usr/local/bin/kubectl \
 && chmod +x /usr/local/bin/kubectl

# Bitwarden CLI (`bw`) — credential lookups against the cluster's
# Vaultwarden (https://warden.leopaska.xyz). Persistent agent
# credentials arrive via SealedSecrets → envFrom; `bw` is here for
# ad-hoc skill-driven lookups (keep `BW_CLIENTID`/`BW_CLIENTSECRET`
# /`BW_PASSWORD` short-lived, set only in the skill that needs them).
ARG BW_VERSION=2026.4.1
RUN ARCH="$(dpkg --print-architecture)" \
 && case "$ARCH" in \
      amd64) BW_ARCH=linux ;; \
      arm64) BW_ARCH=linux-arm64 ;; \
      *) echo "unsupported arch: $ARCH" >&2; exit 1 ;; \
    esac \
 && curl -fsSL "https://github.com/bitwarden/clients/releases/download/cli-v${BW_VERSION}/bw-oss-${BW_ARCH}-${BW_VERSION}.zip" \
        -o /tmp/bw.zip \
 && unzip -q /tmp/bw.zip -d /usr/local/bin \
 && chmod +x /usr/local/bin/bw \
 && rm -f /tmp/bw.zip

# OpenFang binary + upstream agents/ templates from the builder stage
COPY --from=builder /build/target/release/openfang /usr/local/bin/openfang
COPY --from=builder /build/agents                  /opt/openfang/agents

# Workspace skeleton — overlaid by mounted ConfigMaps + PVC at runtime
RUN useradd -m -u 1000 -s /bin/bash openfang \
 && mkdir -p /data/openfang/workspace \
              /data/openfang/skills \
              /data/openfang/agents/vetinari \
              /data/openfang/sessions \
              /data/graphs \
 && chown -R openfang:openfang /data /opt/openfang

USER openfang
WORKDIR /home/openfang

ENV OPENFANG_HOME=/data/openfang \
    RUST_LOG=info,openfang=info

EXPOSE 4200 50051 9100

HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
  CMD curl -fsS http://localhost:4200/api/health || exit 1

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["openfang", "start", "--config=/data/openfang/config.toml"]
