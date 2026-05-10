# Feature Landscape — Orcker

**Domain:** Desktop Docker management for Laravel/TALL Stack devs
**Researched:** 2026-05-10
**Overall confidence:** MEDIUM-HIGH (training data + PRD context; competitive products are stable and well-documented)

---

## 1. Competitive Landscape — What Orcker is NOT

| Tool | Audience | Strength | Why Orcker is Different |
|---|---|---|---|
| **Docker Desktop** | Generic Docker users | Polished container/image/volume browser, K8s, extensions | Generic — no notion of "project", no Laravel awareness, no `.env` editor, no Artisan, no global shared services pattern |
| **Portainer** | Sysadmins / homelabs / multi-host | Multi-host, RBAC, K8s, Swarm orchestration | Server-oriented, web UI, complexity overkill for solo dev local |
| **Lazydocker** | Terminal power users | Fast TUI, low overhead, keyboard-driven | No GUI, no scaffolding, no project-domain concept |
| **OrbStack** | macOS Docker users | Best-in-class Docker runtime + lightweight VMs | macOS-only, focus is the *runtime*, not the *workflow*; Orcker can sit on top of OrbStack |
| **Laravel Sail** | Laravel devs | Official, opinionated `docker-compose` wrapper | CLI-only, one project at a time, no shared services, duplicates Redis/Postgres per project |
| **Laravel Herd** | Laravel devs (Mac/Win) | Native PHP/Nginx, zero-Docker, very fast | Not containerized — diverges from production parity |
| **Laravel Takeout** | Laravel devs | CLI for "global" dev services | Closest spiritual ancestor to Orcker M1, but CLI-only and minimal |
| **Tinkerwell / TablePlus** | Laravel devs | Rich GUI for tinker/db | Adjacent (single tool, not infra) — Orcker complements, not replaces |

**Confidence:** MEDIUM — feature claims based on training data of well-known products; specific version claims should be re-verified at release time.

**Orcker's positioning niche:**
> The visual layer that **Sail/Takeout never built** + the Laravel awareness **Docker Desktop refuses to add** + the "click-to-run" UX **Lazydocker rejects on principle**.

---

## 2. Table Stakes

If these are missing, the user falls back to Docker Desktop + CLI and never returns.

### 2.1 Container Lifecycle (must work flawlessly)

| Feature | Why Expected | Complexity | PRD Mapping |
|---|---|---|---|
| List containers (running/stopped) with status | Baseline of any Docker GUI | Low | M1, M2 (Painel) |
| Start / Stop / Restart container | Universal expectation | Low | M1.05, M2.05 |
| View container logs in real-time (tail -f) | First thing every dev needs when something breaks | Medium (streaming + buffer) | M4.01 |
| Attach/exec into container shell | Debugging baseline | Medium (pty + xterm) | M3.01 |
| Inspect container details (env, mounts, network) | Debug aid | Low | (implicit — needed for M2.05) |
| Container resource usage (CPU/RAM live) | Users expect a "Resources" tab | Medium (stats stream) | M4.02 |

### 2.2 Image / Volume / Network Hygiene

| Feature | Why Expected | Complexity | PRD Mapping |
|---|---|---|---|
| List images with size + age | "Why is my disk full?" | Low | M5.03 |
| Prune unused images / volumes | Disk reclaim is a daily concern | Low | M5.02, M5.03 |
| List volumes with size attribution | Tracing data per project | Medium (size lookup is slow on Docker API) | M5.02 |
| Network listing | Critical because Orcker uses `orcker-global` | Low | M1.01 |

### 2.3 Project / Compose Awareness

| Feature | Why Expected | Complexity | PRD Mapping |
|---|---|---|---|
| Group containers by project / compose file | Anyone with >1 project demands this | Medium (label parsing) | M2 (core abstraction) |
| `docker compose up/down` per project | Universal Laravel workflow | Low | M2.05 |
| Detect existing `docker-compose.yml` on import | Onboarding friction otherwise | Medium (parser + version detection) | M2.02, M5.01 |
| Port-conflict detection | Single most common dev frustration | Medium (port scan + DB of registered ports) | M2.01, R06 |

### 2.4 UX Baseline

| Feature | Why Expected | Complexity | PRD Mapping |
|---|---|---|---|
| Dark mode | 2026 baseline | Low | M7.01 |
| Multi-platform installers (.dmg/.msi/.AppImage) | Distribution table-stakes | Medium (CI matrix) | RNF-06 |
| Auto-updater | Otherwise abandoned within 3 months | Medium (Tauri Updater) | M7, RNF-06 |
| System tray with status | Background-app expectation | Low-Medium | M7.03 |
| Graceful degradation when Docker is down | Crashes break trust immediately | Medium (health probe + UI state) | RNF-04 |

### 2.5 Laravel-Specific Table Stakes

These elevate from "another Docker GUI" to "Laravel tool". Without them, a Laravel dev sees no reason to switch from `sail`.

| Feature | Why Expected | Complexity | PRD Mapping |
|---|---|---|---|
| Run Artisan commands from UI | Daily Laravel workflow | Medium (exec + stream) | M3.03 |
| Tail `storage/logs/laravel.log` alongside Docker logs | Laravel devs check both constantly | Medium (file watcher + multiplex) | M4.01 |
| Visual `.env` editor with diff vs `.env.example` | Universal Laravel pain point | Medium (parser + diff UI) | M2.06 |
| Auto-create `{project}_testing` database | Pest/PHPUnit setup friction | Low (init.sql or runtime SQL) | M6.01 |
| PHP version selector per project | Multi-project Laravel reality | Medium (Dockerfile templating) | M2.01, M5.04 |
| Vite/HMR awareness | Frontend reload is non-negotiable | Medium (port mapping + container process) | M2.03.1 |

---

## 3. Differentiators

These are the reasons a Laravel dev *picks Orcker over Sail + Docker Desktop*. Each is unique vs the competitive set in §1.

### 3.1 Global Stack Pattern (THE differentiator)

| Feature | Value Proposition | Complexity | PRD Mapping |
|---|---|---|---|
| Shared `orcker-global` Docker network | Eliminates duplicate Redis/Postgres/Mailpit instances across projects → less RAM, fewer port conflicts | Medium (network lifecycle + reconnection on container recreate) | M1.01–M1.05 |
| Catalog of global services (Redis, PG, MySQL, Mailpit, MinIO, Soketi, Meilisearch, Typesense) toggleable individually | Zero-config dev infra; Sail equivalent requires manual `docker-compose.override.yml` per project | Medium per service (8 services × config UI) | M1.02–M1.04 |
| One-click Global ON/OFF | Mental model: "I'm starting work" instead of "I'm starting Redis" | Low | M1.05 |

**Competitive moat:** No other tool combines this network + catalog + visual toggle UX. Takeout has the spirit, no UI.

### 3.2 Laravel Stack Scaffolding

| Feature | Value Proposition | Complexity | PRD Mapping |
|---|---|---|---|
| Wizard for new project with stack pick (TALL / Filament v3-v4 / Filament v5 / Inertia+Vue / Inertia+React / API-only / Jetstream) | Hours-to-minutes for new project setup; Sail can't scaffold Filament dirs or detect Livewire/Vite compatibility | High (matrix of stack × file generation, plus compatibility rules — Filament v3/v4 disables Vite HMR, v5 enables it) | M2.03, M5.04 |
| Auto-generated `Dockerfile.dev` + `nginx/` + `php/` + `supervisord.conf` matching production-like reference | Kills the #1 reason devs avoid Docker for Laravel: setup pain | High (templates + variants for compose v2 vs current) | M2.03, M5.04 |
| Marketplace of community templates (GitHub PR-based) | Network-effect feature; competitive moat for OSS adoption | High (schema + indexer + UI) — defer to Phase 2 | M5.06 |

### 3.3 Visual Editors Laravel Devs Have Never Had

| Feature | Value Proposition | Complexity | PRD Mapping |
|---|---|---|---|
| Visual `php.ini` editor (categorized + inline docs + raw fallback) | No competitor offers this for Docker'd PHP | Medium-High (curated directive metadata) | M2.07 |
| Visual Supervisor manager (add/edit/remove workers, numprocs, priority, restart) | Workers are notoriously fragile and CLI-edited; this is a flagship feature for queue-heavy apps (Horizon) | High (config parser + supervisorctl integration) | M2.08 |
| Xdebug toggle + auto-generated `launch.json` (VS Code) / `.idea/` (PhpStorm) | Xdebug setup is universally hated; this is a big-ticket "wow" feature | Medium (mode toggle + IDE detection + config templating) | M2.09 |

### 3.4 Quick Actions / Command Palette

| Feature | Value Proposition | Complexity | PRD Mapping |
|---|---|---|---|
| Pre-mapped Artisan/Composer/NPM/Pest catalog | No "what's the command again?" lookups | Low-Medium | M3.03 |
| `Cmd/Ctrl+K` palette with fuzzy search across all commands | Raycast/VS Code-class UX in a Docker tool | Medium (fuse.js-like + history weighting) | M3.02 |
| Per-project favorites with parameterized templates + numeric shortcuts (`Cmd+1..9`) | Power-user retention | Medium (template engine + binding manager) | M3.04 |
| Command history with re-run from any past execution | Equivalent to a project-aware shell history | Medium (persisted log + UI) | M3.05 |

### 3.5 Sandbox (M8) — Differentiator That No Docker GUI Has

| Feature | Value Proposition | Complexity | PRD Mapping |
|---|---|---|---|
| One-click HTTPS public URL for any project (Cloudflare Tunnel bundled) | Replaces "deploy to staging" for client demos / employer showcases | High (bundled `cloudflared` with SHA-256 verification + lifecycle) | M8.01–M8.03 |
| Bundled QR code (no external API), bcrypt password gate, mandatory expiry, IP allowlist (CIDR), read-only mode | Security-first demo sharing — no other tool combines these | High (proxy middleware + form UI + crypto) | M8.04 |
| Real-time access log with local GeoIP, masked IPs, CSV export | Observability for sandboxed sessions; trust signal for security-conscious users | Medium-High (GeoLite2 bundling + reverse-proxy log stream) | M8.05 |

**Competitive moat:** ngrok/expose are tunnels; Orcker is **a tunnel + a security console + a project context**. No one combines.

### 3.6 Team Collaboration via Git

| Feature | Value Proposition | Complexity | PRD Mapping |
|---|---|---|---|
| `.orcker.json` committed to repo → teammates clone-and-import in seconds | Solves "works on my machine" without DevOps overhead | Medium (schema + import flow + version migration) | M5.05, Persona 2 |
| Git-versioned infra files (`docker-compose.yml`, Dockerfile, etc.) with diff/rollback | No other Docker GUI tracks infra history natively | Medium (git2 integration + diff viewer) | M5.05 |
| Sync-on-open with non-blocking divergence warning | Lets devs edit infra outside Orcker without lockout (PRD Q3) | Medium (file watch + 3-way diff) | M5.05, R09 |

### 3.7 Compose v2 Legacy Support

| Feature | Value Proposition | Complexity | PRD Mapping |
|---|---|---|---|
| Auto-detect `docker-compose` (hyphen, v1.x) vs `docker compose` plugin and adapt syntax | Older Laravel projects (3-5 years old) commonly still use legacy compose; competitors break or ignore | Medium (detection + dual templating) | M2.03, M5.01 |

---

## 4. Anti-Features (Deliberately NOT Built)

These would bloat the app, drag in maintenance, or hurt the target persona. Reject hard.

| Anti-Feature | Why Avoid | What to Do Instead |
|---|---|---|
| **Production / staging env management** | Out of scope per PRD §4; requires SSH, secrets vault, deploy primitives — unbounded scope | Stay dev-local; let Forge/Vapor own production |
| **Cloud deploy integrations (Forge, Vapor, Envoyer)** | Pulls Orcker into deployment-tool category, blurring identity | Out of scope (PRD Q12) — Sandbox covers the demo use case |
| **Kubernetes / Swarm orchestration** | Persona is solo Laravel dev, not platform engineer; massive complexity; Portainer's territory | Single-host Docker only |
| **Multi-host management** | Same reason as K8s; no demand from persona | Local socket only (RNF-03) |
| **Generic non-Laravel project support** | Identity dilution; the visual editors (php.ini, Supervisor) only make sense for PHP | "Laravel/TALL" stays in tagline |
| **Built-in DB GUI (à la TablePlus/DBeaver)** | TablePlus/DBeaver/Sequel Ace already perfect; building a worse version wastes scope | M6 stays at lifecycle (dump/restore/CLI launch); user picks their own GUI |
| **Built-in code editor for app code** | VS Code/PhpStorm exist; Monaco is for *config files only* (compose, php.ini) | Open-in-IDE button |
| **Container image building from scratch (Dockerfile authoring)** | Templates cover 95%; full Dockerfile authoring is Docker Desktop/Buildx territory | Provide templates + raw edit; never wizardize Dockerfile |
| **Plugin marketplace (binary plugins)** | Security nightmare for a tool managing the user's Docker socket | Templates marketplace is GitHub-PR-based, declarative only (M5.06); plugin system deferred to Fase 3 if at all |
| **Built-in CI/CD pipelines** | GitHub Actions / GitLab exist; Orcker is local-only | Out of scope |
| **Account / cloud sync of configs** | Adds backend infra, account system, GDPR concerns; config-via-Git already solves it | M7.04 (file export/import) only |
| **AI assistant / code generation** | Trend-chasing; persona uses external IDE/Claude; bloats binary | Never. Stay focused. |
| **Telemetry / analytics by default** | OSS audience hates this; trust-killer | If ever added: opt-in, anonymous, documented |
| **"Unlimited" sandbox sessions** | Security: forgotten public tunnel = leaked DB | Mandatory expiry (M8.04, RNF-03) — no UI option |
| **Windows ARM / Linux ARM at MVP** | Tauri matrix complexity; persona mostly x64 | Linux ARM Phase 3 only; Windows ARM out of scope |
| **Native Windows containers (Hyper-V)** | Linux containers via WSL2 only; native Windows containers irrelevant for Laravel | WSL2 backend only |
| **In-app billing / paid tiers** | OSS MIT positioning; would require auth + payment infra | Free MIT forever; donations/sponsorship via GitHub if desired |

---

## 5. Feature Dependencies

```
M1 (Global Stack)
  └─ orcker-global network ──────────────┐
                                          │
M2 (Projects)                             │
  ├─ register / scaffold ◄────── M5.04 (templates)
  ├─ join project containers to ─────────┘
  ├─ .env editor ──────► reads project + global service vars
  ├─ php.ini editor ───► restarts PHP-FPM container
  ├─ Supervisor mgr ───► writes supervisord.conf, supervisorctl exec
  └─ Xdebug toggle ────► writes .env + IDE config files

M3 (Quick Actions)
  └─ requires M2 (project context) + container "app" running

M4 (Logs)
  ├─ Docker logs API (any container)
  └─ laravel.log tail (requires project path from M2)

M5 (Infra)
  ├─ M5.01 compose editor ──► writes files M2 reads on next start
  ├─ M5.02–M5.03 volumes/images ─ standalone
  ├─ M5.04 templates ──► consumed by M2.03 wizard
  ├─ M5.05 git versioning ──► requires M2 project + git repo
  └─ M5.06 marketplace ──► consumed by M2.03 wizard

M6 (DB)
  ├─ M6.01 testing DB ──► requires M1 (global Postgres/MySQL) + M2 project
  └─ M6.02 dump/restore ─ requires M1 service running

M7 (Settings)
  └─ standalone; affects all modules visually

M8 (Sandbox)
  ├─ requires M2 project running on local HTTP port
  ├─ requires bundled cloudflared binary (CI-distributed)
  └─ keychain (M7) for ngrok/Expose tokens
```

**Critical-path build order (for roadmap):**
1. Adapters (Docker, FS, Git, Keychain, Shell) — foundation
2. M1 Global Stack — proves the differentiator
3. M2 Projects (register + import; scaffold can phase) — proves the workflow
4. M3 Quick Actions — daily-driver value
5. M4 Logs — closes the debug loop
6. M5 Infra — power-user value
7. M6 DB — completes Laravel workflow
8. M7 Settings + Tray — polish for daily use
9. M8 Sandbox — flagship differentiator (Phase 2 per PRD)

---

## 6. MVP Recommendation (Phase 1 per PRD §11)

**Must ship in v0.1.0 to be "useful":**

1. M1: Global Stack with **Redis + PostgreSQL + Mailpit** (3 services prove the pattern)
2. M2: Register existing project + scaffold **TALL Stack** template (one stack first; Filament/Inertia in Phase 2)
3. M2.03.1 Vite toggle, M2.06 `.env` editor, M2.07 php.ini editor, M2.08 Supervisor, M2.09 Xdebug
4. M3: ~10 most-common Quick Actions (migrate, fresh, tinker, cache:clear, key:generate, queue:work, test, npm dev, composer install, pint)
5. M4.01: Unified log viewer (Docker logs + laravel.log)
6. M5.01: docker-compose editor (Monaco) with v2 legacy detection
7. M6.01: Auto-create `{project}_testing` DB
8. M7: Dark theme, system tray with status, Docker socket config

**Defer to Phase 2:**
- Remaining global services (MySQL, MinIO, Soketi, Meilisearch, Typesense)
- Inertia + Filament templates
- Command palette + favorites + history (M3.02–M3.05)
- Resource dashboard with charts (M4.02)
- Marketplace (M5.06)
- Git-versioned configs with diff/rollback (M5.05 full)
- **All of M8 Sandbox** — flagship feature, but heavy implementation; ship only when M1–M7 are stable

**Defer to Phase 3:**
- Linux ARM64
- Plugin system (if at all)
- Subdomínio orcker.dev (M8.06)
- Telescope integration (M6.03)

---

## 7. Roadmap Implications

| Phase | Theme | Top Features | Pitfalls Likely |
|---|---|---|---|
| **Phase 0 — Done** | Foundation | Tauri scaffold, CI matrix, Release Please | n/a — complete |
| **Phase 1 — MVP Core** | "Solo dev daily driver" | M1 (3 services) + M2 TALL + M3 basics + M4 logs + M6.01 + M7 minimal | Docker socket cross-platform quirks (Colima, OrbStack); Compose v2 legacy parser; xterm.js perf with high-volume logs |
| **Phase 2 — Full Feature** | "Team-ready power tool" | Full M1 catalog, all stack templates, M3 palette/favorites, M5.05 Git + M5.06 marketplace, **all M8 Sandbox** | Cloudflared binary distribution + verification; bcrypt-protected proxy middleware; GeoLite2 licensing/bundling; conflict detection across projects |
| **Phase 3 — Polish & Growth** | "Reach + extension" | Auto-updater polish, ARM64, plugin system (maybe), orcker.dev subdomains | Plugin sandboxing security; DNS infra for orcker.dev |

**Phase ordering rationale:** PRD already prescribes this order; research validates it. The Global Stack (M1) is the differentiator and must ship first to prove the product thesis. Sandbox (M8) is high-value but high-risk (security, bundled binary, CDN); pushing it to Phase 2 lets the security model mature.

**Research flags for phase-specific deep-dives:**
- **Phase 1**: bollard idioms for stats streaming + log streaming under load (verify against current bollard docs at build time)
- **Phase 1**: docker-compose v1 (hyphen) deprecation status and detection heuristic
- **Phase 2**: cloudflared bundling rules per OS (notarization on macOS, signing on Windows, AppImage embedding on Linux)
- **Phase 2**: GeoLite2 license terms (MaxMind requires attribution + license key for download; verify current terms)
- **Phase 2**: middleware-based password gate compatible with all 3 tunnel providers (Cloudflare may need separate strategy than ngrok)

---

## 8. Sources

- **Primary:** Orcker PRD v0.3.1 (`docs/PRD.md`) — 8 modules, 39 RFs, all decisions resolved in §13
- **Primary:** `.planning/PROJECT.md` — validated requirements + constraints
- **Competitive landscape:** Training data on Docker Desktop, Portainer, Lazydocker, OrbStack, Laravel Sail/Herd/Takeout/Tinkerwell — MEDIUM confidence; specific feature claims should be re-validated against current product docs before public marketing copy
- **Reference architecture:** Wallace Martins (Nov 2025) — *Docker para Desenvolvimento Laravel* (cited in PRD §15)
- **Web search:** Not available in this session (permission denied + no Brave API key) — recommendation to re-run feature competitive verification with fresh search before finalizing roadmap commit

**Confidence breakdown:**
- HIGH — anything traceable to PRD (project facts, decisions, scope)
- MEDIUM — competitive claims (well-known products, but features evolve)
- LOW — none asserted; all claims tied to PRD or stable industry knowledge
