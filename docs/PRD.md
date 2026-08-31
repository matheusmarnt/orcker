# PRD — Orcker

> **Produto:** Orcker — orquestrador de ambientes de desenvolvimento PHP/Laravel containerizados (fork do Yerd)
> **Versão do documento:** 1.0 · **Data:** 2026-08-20 · **Status:** aprovado para desenvolvimento
> **Owner:** Matheus Mariano · **Documentos relacionados:** `orcker-analise-viabilidade.md` (v1.1), `orcker-sdd.md`
> **Consumo previsto:** humanos e agentes de codificação (Claude Code). Requisitos possuem IDs estáveis (`FR-xxx`, `NFR-xx`) referenciados pelas specs do SDD — **não renumerar**.

---

## 1. Contexto e problema

Times e desenvolvedores Laravel escolhem hoje entre dois mundos incompletos: ferramentas nativas Herd-like (Herd, Yerd, Valet) com ótima ergonomia (`.test`, HTTPS local, GUI) porém **zero paridade com produção**, e Docker/Sail com paridade porém **sem orquestração multi-projeto** (sem domínios, sem TLS confiável, sem GUI, um compose isolado por projeto). O resultado prático: "funciona na minha máquina", conflitos de versão de PHP e surpresas no deploy.

O Orcker resolve os dois lados de uma vez: fork do Yerd v2 (Rust, MIT) mantendo daemon, proxy TLS com CA local, DNS `.test`, GUI Tauri, CLI, doctor, tunnel e MCP — substituindo o runtime nativo por **Docker + docker compose**, gerando por projeto o stack de paridade de produção do documento de referência (`docs/referencia-docker-laravel.md`): app com PHP-FPM + Supervisor (Horizon, scheduler, Pulse), nginx, banco por projeto, redes duais e serviços globais compartilhados.

## 2. Visão e proposta de valor

> **One-liner:** *"Sail dá containers sem orquestração; Herd dá orquestração sem containers. Orcker dá os dois."*

Proposta de valor em três pilares:

1. **Paridade com produção** — o ambiente local é a topologia real de deploy (FPM + nginx + Supervisor + filas + scheduler), não uma simulação nativa.
2. **Zero lock-in** — tudo que o Orcker gera (`docker/`, `docker-compose.yml`, `Dockerfile.dev`, `orcker.yml`) é versionável e roda com `docker compose up -d` puro, sem o Orcker instalado.
3. **Ergonomia Herd-like** — `<site>.test` com HTTPS de CA local confiável, GUI de bandeja, CLI de primeira classe, doctor, compartilhamento via Cloudflare Tunnel e servidor MCP para agentes de IA.

## 3. Objetivos e métricas de sucesso

| ID | Objetivo | Métrica de sucesso |
|----|----------|--------------------|
| G1 | Ferramenta pessoal de desenvolvimento diária | O próprio owner desenvolve projetos Laravel reais usando exclusivamente o Orcker a partir do fim da Fase 1 |
| G2 | Ativo de portfólio | Repositório público com README de posicionamento, CI verde (Linux + macOS), releases assinados e documentação completa; ≥ 1 artigo/post técnico publicado sobre o projeto |
| G3 | Experiência golden-path rápida | Do `orcker new` ao `https://<site>.test` respondendo 200: ≤ 90 s com imagem base em cache; ≤ 5 min no primeiro uso (pull da imagem) |
| G4 | Qualidade sustentada | Gate de CI (fmt + clippy `-D warnings` + testes) verde em 100% dos merges; cobertura ≥ 80% nas crates novas puras |

## 4. Não-objetivos (fora de escopo do produto)

- **Não é ferramenta de deploy nem de produção** — orquestra exclusivamente ambientes locais de desenvolvimento (os artefatos gerados são "production-shaped", não "production-ready").
- **Não suporta Kubernetes, Swarm ou Podman** no MVP (Docker Engine + docker compose v2 apenas).
- **Não gerencia PHP nativo no host** — a noção de "instalar PHP" do Yerd deixa de existir; versões de PHP são imagens.
- **Windows/WSL2 fora do MVP** (Fase 3; o fork não possui adaptadores Windows).
- **Sem compatibilidade de configuração com Yerd/Herd/Valet** — migração é documentada, não automatizada, no MVP.
- **Sem telemetria/analytics** — herda a postura "local and quiet" do Yerd.

## 5. Personas e jornadas principais

**P1 — Dev Laravel solo (owner).** Cria e mantém N projetos na mesma máquina, alternando versões de PHP e compartilhando Redis/Mailpit/MinIO entre eles.

**P2 — Dev de time que clona um repositório.** O repo já contém `docker/`, `docker-compose.yml` e `orcker.yml` commitados. Com Orcker: `orcker link && orcker up && orcker init` e o site está em `https://app.test`. Sem Orcker: `docker compose up -d` e acesso por porta local — o repositório funciona nos dois mundos.

**P3 — Contribuidor open source.** Adiciona um serviço ao catálogo ou um preset de stack via fragmento tipado + testes, sem tocar no núcleo.

**Jornada golden-path (referência para testes E2E):**

```
orcker new blog --php 8.4 --db postgres   # scaffold + stack de referência
orcker up blog                             # containers sobem, healthchecks ok
# browser: https://blog.test  → 200 (welcome page, cadeado verde)
orcker artisan blog migrate                # migração no banco do projeto
orcker artisan blog tinker                 # REPL interativo
# app envia e-mail → visível no Mailpit; job disparado → processado pelo Horizon
orcker logs blog app                       # logs do container app
orcker eject blog                          # remove dependência do Orcker
docker compose up -d                       # projeto continua funcional standalone
```

## 6. Decisões de produto consolidadas

Decisões validadas com o produto owner em 2026-08-20 (rodadas 1 e 2 — detalhes e trade-offs na análise de viabilidade):

| ID | Decisão |
|----|---------|
| D01 | Fork cirúrgico do Yerd v2 em **Rust** (substituir runtime nativo por backend Docker) |
| D02 | Stack por projeto com **paridade ao documento de referência** (app+Supervisor, nginx, banco, redes duais) |
| D03 | Serviços globais do MVP: **Redis, Mailpit, MinIO, Soketi/Reverb** (rede externa `development`) |
| D04 | Plataformas do MVP: **Linux + macOS** |
| D05 | Banco por projeto: **PostgreSQL + MySQL** (seletor `--db`); MariaDB+ na Fase 3 |
| D06 | Mail padrão: **Mailpit**; sink SMTP herdado vira modo opcional na Fase 2 |
| D07 | Imagens PHP: **híbrido** — base pré-buildada `ghcr.io/<org>/orcker-php:<ver>` por padrão + `--build-from-source` gera o Dockerfile completo do doc de referência |
| D08 | Versões PHP no lançamento: **8.1–8.5**, com rótulo visível de EOL (8.1 sem patches; 8.2 EOL em 31/12/2026) |
| D09 | CLI **híbrida**: `park/link/secure/use` (sites/roteamento) + `up/down/restart/logs/artisan` (ciclo de vida) |
| D10 | Nome público: **Orcker**, com disclaimer de não-afiliação à Docker Inc. |
| D11 | Upstream: **fork silencioso com créditos** (preservar MIT © Forjed, seção Lineage no README) |
| D12 | Estrutura no projeto: **`docker/` + `docker-compose.yml` na raiz + `orcker.yml`** de metadados versionável |

**Premissas adicionais** (assumidas e documentadas): idioma de código, CLI, GUI, docs e mensagens: **inglês** (alcance de portfólio); explicações e artigos de divulgação podem ser bilíngues. Licença do Orcker: **MIT**.

## 7. Requisitos funcionais

Prioridade: **M**ust / **S**hould / **C**ould (MoSCoW). Fase conforme roadmap da análise de viabilidade. Critérios de aceite (AC) são objetivos e verificáveis — as specs do SDD devem mapear cada AC a um teste ou evidência executável.

### E1 — Fundação do fork (Fase 0)

**FR-001 (M, F0) — Rebrand compilável.** Workspace forkado de tag estável do Yerd com crates/binários renomeados (`orcker-*`, `orckerd`, `orcker`, `orcker-helper`) e marca removida.
*AC1:* `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` verdes em Linux e macOS. *AC2:* `orcker ping` responde com daemon rodando. *AC3:* `LICENSE.md` preserva o copyright da Forjed; README contém seção *Lineage/Credits* e disclaimer de não-afiliação (D10, D11).

**FR-002 (M, F0) — Remoção do runtime nativo.** Crates `yerd-php`, `yerd-services` e `yerd-supervise` removidas do grafo de runtime; comandos/mensagens IPC correspondentes removidos ou marcados `deprecated` de forma aditiva.
*AC1:* workspace compila e testa verde sem as três crates. *AC2:* nenhum binário resultante inicia processos PHP/DB nativos.

**FR-003 (M, F0) — Spike de roteamento validado.** Dado um container nginx servindo Laravel (stack do doc de referência montado manualmente), o proxy herdado roteia `https://spike.test` para ele.
*AC1:* `orcker link` registra o site com upstream HTTP `127.0.0.1:<porta>`. *AC2:* browser recebe 200 com certificado da CA local válido. *AC3:* WebSocket upgrade funcional (Vite HMR conecta).

### E2 — Engine Docker (Fase 1)

**FR-010 (M, F1) — Detecção do ambiente Docker.** O daemon detecta socket do Docker, versão do Engine, presença/versão do compose v2 e contexto ativo; expõe via IPC e `orcker status`.
*AC1:* com Docker parado, `orcker status` reporta o problema com próximo passo acionável (sem panic). *AC2:* saída `--json` inclui `docker.engine_version`, `docker.compose_version`.

**FR-011 (M, F1) — Ciclo de vida compose.** `up`, `down`, `restart` e `build` por projeto executados via `docker compose` CLI (o compose file é o contrato), com erros tipados e timeout configurável.
*AC1:* `orcker up <site>` retorna somente após healthchecks do compose satisfeitos ou timeout com erro tipado. *AC2:* falha de build/pull produz mensagem com causa e comando de reprodução manual.

**FR-012 (M, F1) — Observação via Engine API.** Estado por container (running/health/exit code) e eventos consumidos via [bollard](https://github.com/fussybeaver/bollard) e refletidos no estado do daemon (broadcast aos clientes).
*AC1:* matar um container externamente (`docker kill`) atualiza `orcker status` e a GUI em ≤ 5 s sem reinício do daemon.

**FR-013 (M, F1) — Alocação de portas loopback.** Porta efêmera por projeto (nginx→host) alocada sem colisão, publicada apenas em `127.0.0.1`, persistida e reusada entre restarts.
*AC1:* dois projetos simultâneos recebem portas distintas e estáveis após `orcker restart`. *AC2:* nenhuma porta de projeto é exposta em `0.0.0.0` nos composes gerados.

### E3 — Geração de stack (Fase 1)

**FR-020 (M, F1) — `orcker new`.** `orcker new <name> --php <8.1..8.5> [--db postgres|mysql] [--preset reference|minimal] [--build-from-source]` cria projeto Laravel (via container, sem Composer no host) + stack completo (D02, D05, D07).
*AC1:* golden-path da seção 5 passa de ponta a ponta. *AC2:* `--db mysql` produz init script e healthcheck próprios do MySQL. *AC3:* sem flag, `Dockerfile.dev` é fino (`FROM ghcr.io/<org>/orcker-php:<ver>`); com `--build-from-source`, é o Dockerfile completo do doc de referência.

**FR-021 (M, F1) — `orcker link`.** Adota projeto existente: detecta webroot/framework (detector herdado), gera `docker/` + compose + `orcker.yml` quando ausentes e apenas registra quando presentes (idempotente).
*AC1:* repo clonado com artefatos commitados: `orcker link` não altera nenhum arquivo existente e o site sobe. *AC2:* projeto sem artefatos: gera tudo e imprime resumo do que criou.

**FR-022 (M, F1) — Templates com paridade ao doc de referência.** Templates renderizados por crate pura: compose (redes duais, `restart: "no"`, healthcheck no banco, volume `:ro` do storage no nginx, porta Vite 5173), nginx (global + vhost com `FASTCGI_PASS` templated), `php.ini` (opcache, limites, timezone configurável), `supervisord.conf` com programas **toggláveis por projeto** (`php-fpm`, `horizon`, `schedule`, `pulse` — D08/nuance de legado), init de banco por engine (timezone, locale, banco de testes), `entrypoint.sh`, `orcker.yml`.
*AC1:* snapshot tests dos templates cobrem as combinações `{postgres,mysql} × {reference,minimal} × {fino,source}`. *AC2:* compose gerado valida contra `docker compose config` sem warnings. *AC3:* UID/GID do host aplicados no build (arquivos criados no container pertencem ao usuário).

**FR-023 (S, F1) — Regeneração explícita.** Arquivos gerados pertencem ao usuário após a criação; `orcker stack regenerate [--diff]` mostra/aplica atualizações de template sob demanda (nunca sobrescrita silenciosa).
*AC1:* `--diff` não modifica arquivos; aplicar exige confirmação ou `--force`.

**FR-024 (M, F1) — Schema do `orcker.yml`.** Metadados versionados (`schema_version`, site, PHP, engine de banco, preset, serviços usados, programas do Supervisor ativos), com parser tolerante a campos desconhecidos (forward-compat).
*AC1:* `orcker link` em máquina B reproduz a configuração da máquina A somente a partir do repo. *AC2:* schema documentado em `docs/reference/orcker-yml.md`.

### E4 — Sites e roteamento (Fase 1)

**FR-030 (M, F1) — `park`/`unpark`/`link`/`unlink`** com domínio `<name>.test` automático (DNS embutido herdado).
*AC1:* pasta parkeada: novo subdiretório com projeto linkável aparece em `orcker sites` e resolve no browser após `up`.

**FR-031 (M, F1) — Proxy HTTP + WebSocket.** Roteamento `<site>.test` → upstream HTTP do nginx do projeto, com upgrade de WebSocket (Vite HMR e Soketi/Reverb).
*AC1:* `npm run dev` no container: HMR atualiza o browser sem reload manual. *AC2:* canal de broadcast Laravel Echo conecta via `wss://<site>.test`.

**FR-032 (M, F1) — `secure`/`unsecure`** por site com CA local herdada (leaf on demand).
*AC1:* cadeado válido no Chrome/Firefox após `sudo orcker elevate trust` one-shot; HTTP→HTTPS conforme configuração do site.

**FR-033 (S, F1) — Fallback localhost herdado.** Sem elevação, sites acessíveis via `http://localhost:8080/~<name>.test`.
*AC1:* funcional para site em container (teste de regressão do comportamento herdado).

### E5 — Ciclo de vida e operação (Fase 1)

**FR-040 (M, F1) — `up`/`down`/`restart`/`status`** por projeto e agregados (`orcker status` global mostra projetos + serviços globais + saúde).
**FR-041 (M, F1) — `orcker logs <site> [service] [-f]`** com merge/cores por serviço.
**FR-042 (M, F1) — `orcker artisan <site> <args…>` e `orcker exec <site> [service] <cmd…>`** (TTY interativo funcional — `tinker` é o AC).
**FR-043 (M, F1) — `orcker init <site>`.** Executa a sequência de inicialização do doc de referência dentro do container: `composer install`, `.env` (copy + `key:generate`), `migrate [--seed]`, `npm install && npm run build`.
*AC (E5):* golden-path completo; `orcker artisan blog tinker` abre REPL utilizável; `logs -f` acompanha em tempo real.

### E6 — Serviços globais compartilhados (Fase 1)

**FR-050 (M, F1) — Rede `development`.** Criada/verificada pelo daemon; composes de projeto a referenciam como `external: true`.
**FR-051 (M, F1) — Gestão dos serviços do MVP.** `orcker services` + `service install|start|stop|restart|logs|set-port` para **redis, mailpit, minio, soketi** com versões pinadas no catálogo.
**FR-052 (M, F1) — Integração `.env`.** Projetos gerados apontam `REDIS_HOST=redis`, `MAIL_HOST=mailpit`, `AWS_ENDPOINT=http://minio:9000`, broadcast conforme preset (D03, D06).
**FR-053 (S, F1) — Catálogo tipado extensível.** Serviços definidos como fragmentos compose tipados e validados na crate de catálogo; adicionar serviço novo não toca no núcleo.
*AC (E6):* e-mail enviado pela app aparece na UI do Mailpit; job em fila Redis processado pelo Horizon do container app; upload via Scout/S3 driver chega ao MinIO; evento broadcast chega ao browser via Soketi.

### E7 — Versões de PHP (Fase 1)

**FR-060 (M, F1) — `orcker use <site> <ver>`.** Troca a versão PHP do projeto (atualiza `orcker.yml` + build arg / tag da imagem) e reconstrói mediante confirmação.
**FR-061 (M, F1) — Faixa 8.1–8.5 com rótulo EOL** (D08) em `orcker list php` e na GUI (8.1 `EOL`, 8.2 `security-only até 2026-12-31`, dinâmico por data).
**FR-062 (S, F1) — `orcker list php`** distingue imagens presentes localmente vs disponíveis no registry.
*AC (E7):* alternar 8.4→8.3 num projeto e `php -v` no container reflete a mudança; rótulos EOL corretos na data do teste.

### E8 — Doctor (Fase 1)

**FR-070 (M, F1) — Checks Docker-aware.** Além dos herdados (CA trust, resolver `.test`, portas 80/443): Docker Engine acessível, permissão no socket (grupo `docker` no Linux), compose v2 presente, rede `development` existente, espaço em disco para imagens, colisões de porta.
**FR-071 (S, F1) — `orcker doctor fix`** aplica correções seguras (criar rede, sugerir — nunca executar — adição ao grupo docker).
*AC (E8):* em máquina limpa sem Docker, `doctor` lista exatamente o que falta com comandos de correção; com ambiente ok, zero findings.

### E9 — GUI desktop (Fase 1)

**FR-080 (M, F1) — Páginas adaptadas.** *Projects* (lista com estado por container, ações up/down/restart/logs, PHP e banco do projeto), *Services* (globais, com estado e portas), *PHP/Images* (versões, EOL, pull/rebuild), *Doctor*, *Settings*. Página *Mails* abre a UI do Mailpit.
**FR-081 (S, F1) — Onboarding guiado** adaptado ao fluxo Docker (elevate → docker check → primeiro projeto).
**FR-082 (C, F2) — Viewer de logs embutido** na GUI (stream por container).
*AC (E9):* toda ação da GUI tem equivalente CLI e ambos refletem o mesmo estado do daemon (regra herdada "one source of truth"); `npm run test && npm run build` verdes.

### E10 — Eject / zero lock-in (Fase 1)

**FR-090 (M, F1) — `orcker eject <site>`.** Gera override standalone (portas publicadas, instruções de acesso), remove o site do registro do daemon e imprime o que mudou; o projeto continua subindo com `docker compose up -d` puro.
*AC1:* após eject, com o daemon parado, `docker compose up -d && curl -s http://localhost:<porta>` responde 200. *AC2:* eject é reversível com `orcker link`.

### E11 — MCP para agentes de IA (Fases 1–2)

**FR-100 (S, F1) — MCP herdado funcional.** `orcker mcp` serve o catálogo herdado adaptado (sites, status, doctor, mail via Mailpit URL).
**FR-101 (S, F2) — Tools Docker-aware.** `stack_up`, `stack_down`, `container_logs`, `artisan_run` com gating opt-in herdado.
*AC (E11):* Claude Code conectado ao MCP lista tools e executa `status` end-to-end.

### E12 — Pipeline de imagens (Fase 1)

**FR-110 (M, F1) — Imagens base publicadas.** CI (GitHub Actions) builda e publica `ghcr.io/<org>/orcker-php:{8.1..8.5}` para `linux/amd64` e `linux/arm64`, com as extensões do doc de referência, Composer, Node LTS e Supervisor; tags imutáveis por digest + tag móvel por minor.
**FR-111 (S, F2) — Scan e procedência.** Trivy/grype no CI; SBOM e assinatura (cosign) na Fase 2/3.
*AC (E12):* `docker pull` das 10 combinações funciona; imagem roda `php -m` com todas as extensões da tabela do doc de referência; workflow público no repo.

### E13 — Capacidades herdadas: regressão e adaptação (Fases 1–2)

**FR-120 (S, F1) — Tunnel.** `orcker tunnel share <site>` publica site containerizado (regressão do fluxo herdado apontando para o proxy local).
**FR-121 (S, F2) — Sink SMTP herdado opcional** (D06): toggle por projeto injeta `MAIL_HOST=host.docker.internal` + `extra_hosts` no compose.
**FR-122 (C, F2) — Dumps/telemetria em container.** Extensão de dumps instalada na imagem (`--with-dumps`), emitindo para o daemon via `host.docker.internal` (host-gateway).
*AC (E13):* FR-120: URL `*.trycloudflare.com` responde. FR-121/122: fluxo funcional documentado com limitações.

### E14 — Documentação (Fase 1)

**FR-130 (M, F1) — Docs de produto.** README com posicionamento + quickstart; site de docs (VitePress herdado) com guia de instalação, golden-path, referência CLI (`--json` incluso), referência `orcker.yml`, guia de migração (Sail/Herd/Yerd) e página de arquitetura para contribuidores.
*AC1:* um dev sem contexto executa o quickstart em macOS e Linux sem apoio externo. *AC2:* toda flag documentada existe e todo exemplo roda (docs testadas no gate quando viável).

## 8. Requisitos não-funcionais

| ID | Requisito | Critério verificável |
|----|-----------|----------------------|
| NFR-01 | **Segurança / privilégios**: daemon e GUI nunca rodam como root; `orcker-helper` permanece a única superfície privilegiada (CA, resolver, portas 80/443); templates jamais geram containers `privileged`, portas em `0.0.0.0` ou montagem do socket Docker em containers | Revisão nos snapshot tests dos templates + testes herdados do helper |
| NFR-02 | **Performance**: `orcker up` ≤ 30 s com imagens em cache (p95, projeto de referência); overhead do proxy ≤ 5 ms p95 local; daemon idle ≤ 50 MB RSS | Benchmarks reproduzíveis em `scripts/bench/` |
| NFR-03 | **Qualidade**: gate herdado obrigatório (fmt, clippy `-D warnings`, testes, GUI build) em CI Linux + macOS; cobertura ≥ 80% linhas nas crates novas puras (`orcker-stack`, `orcker-catalog`) | CI bloqueante; relatório de cobertura publicado |
| NFR-04 | **Compatibilidade**: Docker Engine ≥ 24 e docker compose v2 (mínimos exatos fixados na Fase 0 e validados pelo doctor); Linux x86_64/arm64; macOS Apple Silicon (Docker Desktop, OrbStack e colima documentados) | Matriz testada no CI + doctor |
| NFR-05 | **Disciplina arquitetural herdada**: lógica pura sem I/O; side effects atrás de traits com fakes; sem `unsafe`; sem `unwrap`/`panic!` fora de testes; `thiserror` em libs; IPC evolui apenas aditivamente | Lints do workspace + wire-stability tests |
| NFR-06 | **Licenciamento**: MIT com aviso © Forjed preservado; NOTICE/Credits no README; zero assets de marca do Yerd no produto final | Checklist de release |
| NFR-07 | **Privacidade**: nenhuma chamada de rede além das explicitamente solicitadas (pull de imagens, tunnel, update check notify-only) | Auditoria de dependências + política herdada |
| NFR-08 | **UX de erro**: toda falha voltada ao usuário inclui causa + próximo passo acionável; `--json` disponível em todos os comandos | Testes de snapshot das mensagens principais |

## 9. Restrições e dependências

Upstream congelado em tag estável do Yerd registrada em `docs/UPSTREAM.md` (data, tag, commit); Docker Engine/compose como dependências de runtime do usuário (nunca embutidas); imagens oficiais `php:*-fpm` (Debian) como base das `orcker-php`; GHCR como registry; toolchain Rust pinada (herdada); Tauri v2 + Vue 3 na GUI. Publicação de release exige CI verde nas duas plataformas.

## 10. Critérios de lançamento do MVP (release gate da Fase 1)

O MVP é liberável quando, e somente quando:

1. Golden-path (seção 5) passa **de ponta a ponta em Linux e macOS**, gravado como teste E2E reproduzível + demo em vídeo/GIF no README.
2. Todos os FR **Must** da Fase 1 com ACs verdes; **Should** da Fase 1 ≥ 80% concluídos ou re-fasеados com justificativa registrada.
3. `orcker doctor` sem findings em máquina limpa provisionada só com Docker + Orcker.
4. Imagens `orcker-php` 8.1–8.5 publicadas nas duas arquiteturas (FR-110).
5. NFR-01, NFR-03, NFR-06 auditados no checklist de release; demais NFRs sem regressão conhecida.
6. Docs do FR-130 publicadas; versão `0.x` etiquetada com changelog.

## 11. Riscos (resumo)

Detalhamento e mitigação na análise de viabilidade, seção 8: proficiência Rust (mitigada por harness agêntico + crates novas puras primeiro), drift do upstream (tag congelada + mudanças concentradas em crates novas), marca Orcker×Docker (disclaimer + monitoramento), integrações herdadas que dependem de loopback (host-gateway, Fase 2), performance de volumes no macOS (documentar OrbStack/virtiofs).

## 12. Governança do documento e rastreabilidade

Este PRD é a fonte de verdade de **o quê**; o SDD define **como** se implementa. Toda spec (`specs/SPEC-xxxx-*.md`) declara `covers: [FR-…/NFR-…]`; a matriz de rastreabilidade FR ↔ spec ↔ testes vive em `specs/TRACEABILITY.md` e é atualizada a cada ciclo aceito (obrigação do loop, ver SDD §6). Mudanças de requisito exigem nova versão do PRD (minor para adições aditivas, major para mudanças de decisão D01–D12) — agentes **não** editam o PRD; propõem mudanças via `docs/rfc/`.

## 13. Referências

- Análise de viabilidade Orcker v1.1 (documento irmão) · Yerd: https://github.com/forjedio/yerd · Artigo do stack de referência: https://wallacemartinss.dev/pt/blog/docker-for-laravel-development
- Docker Compose: https://docs.docker.com/compose/ · Dockerfile: https://docs.docker.com/reference/dockerfile/ · Engine API: https://docs.docker.com/reference/api/engine/ · bollard: https://docs.rs/bollard
- Laravel 12.x: https://laravel.com/docs/12.x · Horizon: https://laravel.com/docs/12.x/horizon · Reverb: https://laravel.com/docs/12.x/reverb · Scout: https://laravel.com/docs/12.x/scout
- Suporte PHP: https://www.php.net/supported-versions.php · GHCR: https://docs.github.com/packages · Mailpit: https://github.com/axllent/mailpit · MinIO: https://min.io · Soketi: https://github.com/soketi/soketi
