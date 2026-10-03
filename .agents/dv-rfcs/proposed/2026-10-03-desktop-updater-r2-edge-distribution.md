# DV-RFC: Desktop updater distribution via Cloudflare R2 edge

Status: proposed

## Problem

Every shipped desktop installer bakes the Tauri updater endpoint into its binary (`plugins.updater` in `apps/desktop/src-tauri/tauri.conf.json`); once an installer is in a user's hands, the endpoint can only be changed by shipping an update through it. The endpoint the repo baked (`releases.dropvoice.app`) named a domain the implemented public topology ([2026-08-30](../implemented/2026-08-30-deterministic-three-host-public-topology.md)) never defined — no host, no manifest generator — and builds lacked `createUpdaterArtifacts`, so no signed updater artifacts existed either: the update chain was dead at three links. Before the first public install (the trial operation), the project must decide where updates are served from: worldwide reach (zh is a first-class locale), zero consumption of the 3 Mbps origin VPS, and no growth of the ansible-managed origin surface.

## Proposal

Serve the updater distribution from Cloudflare R2 under `releases.dropvoice.online` — a custom domain on the existing `dropvoice.online` zone, edge-cached, zero egress fees. On a stable tag, a new `publish-r2` job in `release.yml` downloads the GitHub release assets (tauri-action's `latest.json` plus the signed installers), rewrites each platform URL to `<domain>/files/<tag>/<filename>` (`scripts/publish_updater_manifest.py`), and uploads via wrangler: artifacts `immutable`, `update/manifest.json` `no-cache`. Prerelease tags never move the manifest through CI — the same stability rule that governs the docker `latest` tag in `docker-publish.yml`; a manual `--allow-prerelease` escape hatch on the publish script exists solely to rehearse the update chain before the first stable ships (while no install base exists to surprise). The orphaned pubkey in `tauri.conf.json` is replaced with a fresh minisign keypair (private key lives outside the repo, delivered as the `TAURI_SIGNING_PRIVATE_KEY` secret); with zero installed clients the rotation is free now and impossible after the first ship. Dollar cost at trial scale is zero — the R2 free tier (10 GB-month storage, 1M Class A / 10M Class B operations, free egress) dwarfs a ~100 MB release — but enabling R2 requires a payment method on the account.

## Alternatives considered

**GitHub Releases as the updater endpoint** (`…/releases/latest/download/latest.json`; zero new infrastructure, since tauri-action already uploads `latest.json`). It lost on reachability: github.com download URLs are unreliable from mainland China, and zh is a first-class locale. It also cedes availability and rate limits to a third party, with no cache headers of our own.

**A fourth host on the production VPS serving static files.** It lost because it couples release distribution to the signaling origin — a bad deploy or a download spike would damage pairing — and spends the 3 Mbps origin link the three-host topology exists to protect, all for pure static bytes that Cloudflare's edge serves for free.

**Do nothing (leave the endpoint dead).** It lost because every update would require a manual reinstall while the wrong endpoint stays baked into each shipped installer — forfeiting the one mechanism that can migrate installed clients, at exactly the moment fixing it becomes expensive (an update through a dead endpoint is the only migration path).

## Acceptance criteria

- A release build produces `.sig` files and a `latest.json` (`createUpdaterArtifacts` on, signing secret present).
- After a stable tag, `https://releases.dropvoice.online/update/manifest.json` returns the new version with complete platform entries; every `url` resolves (200, `immutable`) and every `signature` is its `.sig` content.
- A prerelease tag leaves the manifest untouched through CI; `--allow-prerelease` is the only path by which a prerelease moves it, deliberate and loud.
- An installed older build shows the update toast and updates end-to-end through the endpoint.

## Risks

- R2 requires a payment method to enable, and usage beyond the free tier is metered (Class B reads $0.36 per million) — exposure is bounded and far away at trial scale, but it is not a hard-capped zero.
- The new minisign private key is single-shot infrastructure: lost before the first ship it rotates freely; lost after, installed clients can never be updated again. It must be backed up (password manager) in the same change that creates it.
- The pipeline depends on the `CLOUDFLARE_API_TOKEN` secret (Account → R2 → Edit); a missing token fails `publish-r2` loudly after the GitHub release already exists — recovery is rerunning the job or the script by hand.
