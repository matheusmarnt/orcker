# Session Resume — Orcker

> Atualizar ao fim de cada sessão. Carregado automaticamente via CLAUDE.md.

## Estado Atual

```
PHASE:       1 — Foundation (READY TO PLAN)
NEXT_CMD:    /gsd:plan-phase 1
VERSION:     0.1.0 (Release Please PR mergeado — release em criação)
LAST_COMMIT: docs: add STATE.md — project memory and phase tracker
OPEN_PRS:    none
BLOCKED_BY:  none
```

## Próxima Sessão — Checklist

- [ ] Verificar `reka-ui` vs `radix-vue` como peer dep atual do shadcn-vue (antes de instalar)
- [ ] Verificar versão atual `tauri-specta` compatível com Tauri 2 minor instalado
- [ ] Gerar + fazer backup do Ed25519 keypair do Tauri updater (P-M3: unrecoverable se perdido)
- [ ] Verificar bollard latest minor (0.17 vs 0.18)

## GSD Planning State

```
.planning/
  RESUME.md          ← este arquivo
  STATE.md           ← estado detalhado de fases e decisões
  REQUIREMENTS.md    ← fonte de verdade (R-A.* até R-M8.* + RNF)
  ROADMAP.md         ← 5 fases mapeadas com requisitos e versões
  PROJECT.md         ← contexto, stack, decisões arquiteturais
  research/
    SUMMARY.md       ← síntese executiva de 4 dimensões de pesquisa
    STACK.md         ← padrões bollard, Channel<T>, tauri-specta, erros
    ARCHITECTURE.md  ← 4 camadas obrigatórias, event-sourced cache
    FEATURES.md      ← diferenciadores, anti-features, MVP cut
    PITFALLS.md      ← P-C1..P-C8 críticos, P-M2/M3 release
  codebase/
    STACK.md ARCHITECTURE.md STRUCTURE.md
    CONVENTIONS.md TESTING.md INTEGRATIONS.md CONCERNS.md
```

## Decisões Permanentes

| Decisão | Motivo |
|---------|--------|
| `initial-version: "0.1.0"` em release-please-config.json | Sem isso, `0.0.0` → release-please propõe `1.0.0` |
| `Channel<T>` para IPC streams, `invoke` para one-shot | Backpressure, sem drops em bursts |
| `docker compose` shell-out (não bollard) para Compose | bollard não implementa Compose |
| `tokio::sync::Mutex` (NUNCA `std::sync::Mutex`) em AppState | Evita deadlock cross-.await |
| tauri-specta para type bridge Rust↔TS | 39+ RFs = drift garantido sem codegen |
| M8 Sandbox — último, Phase 5 | Superfície de segurança máxima; requer fundação madura |

## Roadmap Resumido

| Phase | Meta | Versão |
|-------|------|--------|
| 0 | Scaffold + CI | baseline ✓ |
| 1 | Foundation: bollard, AppState, AppError, tauri-specta, tracing | — |
| 2 | M1 Global Stack (Redis/PG/Mailpit) — prova o thesis | v0.1.0 |
| 3 | M2 + M3 + M4 — daily driver completo | v0.2.0 |
| 4 | M5 + M6 + M7 — power tools | v0.3.0 |
| 5 | M8 Sandbox — cloudflared, bcrypt, GeoIP | v1.0.0 |
