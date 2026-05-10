# PRD — Orcker: DevStack Manager
### Product Requirements Document · Versão 0.3

> **Status:** Refinado — M8 Sandbox incorporado · Pronto para SAD (Software Architecture Document)
> **Autor:** Solo Developer
> **Data:** 2025
> **Classificação:** Interno — Planejamento de Produto
> **Domínio:** orcker.dev
> **Licença:** MIT (Open-source)
> **Repositório:** GitHub público desde o MVP

---

## Índice

1. [Visão Geral do Produto](#1-visao-geral-do-produto)
2. [Problema e Oportunidade](#2-problema-e-oportunidade)
3. [Objetivos e Métricas de Sucesso](#3-objetivos-e-metricas-de-sucesso)
4. [Público-Alvo e Personas](#4-publico-alvo-e-personas)
5. [Escopo Funcional — Módulos e Requisitos](#5-escopo-funcional)
   - M1 Global Stack · M2 Projetos · M3 Terminal · M4 Logs · M5 Infra · M6 Banco · M7 Configurações · **M8 Sandbox**
6. [Requisitos Não-Funcionais](#6-requisitos-nao-funcionais)
7. [Arquitetura de Alto Nível](#7-arquitetura-de-alto-nivel)
8. [UX/UI — Diretrizes e Princípios](#8-uxui--diretrizes-e-principios)
9. [Modelo de Dados Lógico](#9-modelo-de-dados-logico)
10. [Stack Recomendada — Justificativa Técnica](#10-stack-recomendada)
11. [Roadmap e Faseamento (MVP → GA)](#11-roadmap-e-faseamento)
12. [Riscos, Restrições e Mitigações](#12-riscos-restricoes-e-mitigacoes)
13. [Decisões de Produto — Refinamento §13 (Resolvido)](#13-perguntas-para-refinamento)
14. [Glossário Técnico](#14-glossario-tecnico)
15. [Referências](#15-referencias)

---

## 1. Visão Geral do Produto

### 1.1 Nome e Tagline

**Orcker** — *Your Laravel Dev Infrastructure, Under Control.*

### 1.2 Definição

O **Orcker** é uma aplicação desktop multiplataforma (Linux, macOS, Windows) voltada exclusivamente para desenvolvedores Laravel que utilizam a **TALL Stack** (TailwindCSS, AlpineJS, Livewire, Laravel) com Vite, **Inertia.js** e **Filament** (v3/v4 com Livewire v3/v3.5 e v5 com Livewire v4). Seu objetivo central é ser a **central de comando** para gerenciamento de infraestrutura de desenvolvimento em contêineres Docker — tanto em nível global (serviços compartilhados entre todos os projetos) quanto em nível de projeto (serviços específicos por aplicação).

A ferramenta elimina a necessidade de o desenvolvedor interagir diretamente com CLI do Docker, arquivos `docker-compose.yml` e scripts shell fragmentados, oferecendo uma interface visual moderna, intuitiva e segura que centraliza todas as operações do ciclo de desenvolvimento. O Orcker suporta **times pequenos** via compartilhamento de configuração através de Git, garantindo paridade de ambiente entre todos os membros sem configuração manual.

### 1.3 Posicionamento de Mercado

| Ferramenta | Abordagem | Limitação |
|---|---|---|
| **Laravel Sail** | Wrapper Docker opinativo para Laravel | Sem gerência visual, sem controle de infra global, um projeto por vez |
| **Docker Desktop** | UI genérica para Docker | Não orientado a Laravel, sem automação de stack, curva alta |
| **Herd (Laravel)** | Dev server nativo sem Docker | Não containerizado, sem equivalência com produção |
| **Orcker** | UI especializada + infra global + automação TALL | — (proposta deste PRD) |

### 1.4 Proposta de Valor

- **Um clique** para ligar/desligar toda a infraestrutura de desenvolvimento.
- **Serviços globais compartilhados** (Redis, PostgreSQL, MySQL, Mailpit, MinIO, Soketi) servindo múltiplos projetos simultaneamente.
- **Serviços por projeto** adicionados individualmente sem afetar os demais.
- **Atalhos e favoritos** para as operações mais comuns do ciclo Laravel.
- **Hot-reload Vite configurável por projeto** — no container `app` (porta 5173), iniciado automaticamente ao ligar o projeto ou sob demanda manual, conforme preferência do dev.
- **Suporte nativo a TALL Stack, Filament e Inertia.js** desde o MVP, com templates específicos para cada abordagem e detecção automática de compatibilidade Livewire/Vite por versão do Filament.
- **Colaboração em times** via sincronização de configuração de infra por Git — paridade de ambiente garantida entre todos os membros.
- **Arquivos de infra editáveis livremente** fora do Orcker — o Orcker sincroniza ao abrir e avisa sobre divergências, sem bloquear o fluxo do dev.
- **Sandbox de demonstração integrado** — expõe qualquer projeto local à internet via túnel reverso HTTPS com um clique, com proteção por senha, expiração automática e log de acesso em tempo real; ideal para homologação com clientes e validação com empregadores.
- **Open-source (MIT)** — 100% gratuito, repositório público no GitHub desde o MVP, contribuições da comunidade bem-vindas.
- **Interface visual profissional** com estética tecnológica, dark-first e design system consistente.

---

## 2. Problema e Oportunidade

### 2.1 Problemas Identificados

#### P1 — Fragmentação do ambiente de desenvolvimento
O desenvolvedor Laravel mantém múltiplos projetos ativos. Cada projeto possui seu próprio `docker-compose.yml`, com serviços duplicados (Redis, Mailpit, PostgreSQL). Resultado: múltiplas instâncias do mesmo serviço consumindo memória e CPU, conflitos de porta, e dificuldade de gerenciar qual serviço pertence a qual projeto.

**Evidência técnica:** O documento de referência (Wallace Martins, 2025) já demonstra a solução arquitetural — redes Docker compartilhadas (`development`) para serviços globais — mas essa configuração exige hoje habilidade avançada em Docker para ser implementada e mantida manualmente.

#### P2 — Overhead cognitivo e operacional
Operações rotineiras do dia a dia de desenvolvimento Laravel exigem múltiplos comandos CLI:
```bash
docker compose exec app php artisan migrate
docker compose exec app php artisan tinker
docker compose exec app ./vendor/bin/pest
docker compose logs -f app
```
Esse overhead é multiplicado por projeto e por frequência de uso — dezenas de execuções por dia.

#### P3 — Inconsistência entre projetos
Sem uma ferramenta centralizadora, cada projeto acumula configurações divergentes de Docker, versões diferentes de serviços, e convenções ad-hoc. A ausência de um padrão causa retrabalho na incorporação de novos projetos.

#### P4 — Ausência de observabilidade
O desenvolvedor não tem visibilidade imediata do estado dos serviços: quais estão rodando, qual está consumindo mais CPU/memória, quais logs estão gerando erros — sem abrir múltiplos terminais ou acessar o Docker Desktop genérico.

#### P5 — Curva de entrada para novos projetos
Configurar do zero um ambiente Docker completo para um novo projeto Laravel (Dockerfile, Nginx, Supervisor, PHP-FPM, PostgreSQL, Healthchecks, redes) consome horas e exige conhecimento avançado que vai além do desenvolvimento da aplicação em si.

### 2.2 Oportunidade

A combinação de:
- Crescimento acelerado do ecossistema Laravel (TALL Stack, Livewire 4, Laravel Reverb, Inertia.js, Filament v5)
- Adoção crescente de Docker em desenvolvimento local
- Ausência de uma ferramenta especializada de gerência visual para esse nicho
- Modelo open-source MIT — zero barreira de adoção, comunidade como motor de crescimento e validação

...cria uma janela clara para um produto focado e de alto valor para o desenvolvedor Laravel solo ou em times pequenos.

---

## 3. Objetivos e Métricas de Sucesso

### 3.1 Objetivos de Negócio

| OBJ | Descrição |
|---|---|
| OBJ-01 | Ser a única ferramenta necessária para gerência da infra de dev Docker de um desenvolvedor Laravel |
| OBJ-02 | Reduzir o tempo de setup de um novo projeto Laravel dockerizado de horas para minutos |
| OBJ-03 | Eliminar a necessidade de conhecimento avançado de Docker para operar o ambiente de desenvolvimento |
| OBJ-04 | Ser adotável por times pequenos com paridade total de ambiente via Git, sem configuração manual |
| OBJ-05 | Construir uma ferramenta open-source de referência para o ecossistema Laravel, fomentando contribuições da comunidade |
| OBJ-06 | Permitir que o desenvolvedor apresente seu trabalho local a clientes e empregadores sem necessidade de deploy em ambiente remoto |

### 3.2 Métricas de Sucesso (KPIs — Pós-Lançamento)

| Métrica | Meta MVP | Meta 6 meses |
|---|---|---|
| Tempo para ter um novo projeto rodando | < 3 min | < 90 seg |
| Comandos CLI executados pelo usuário por sessão | < 5 | < 2 |
| Retenção de uso semanal | — | > 70% |
| Número de projetos gerenciados simultaneamente sem conflito | ≥ 5 | ≥ 15 |
| Cobertura de testes do core | ≥ 80% | ≥ 90% |

---

## 4. Público-Alvo e Personas

### Persona Primária — "O Dev Solo Laravel"

| Atributo | Descrição |
|---|---|
| **Perfil** | Desenvolvedor web freelancer ou indie, 2-8 anos de experiência |
| **Stack** | Laravel 11/12, Livewire 3/4, TailwindCSS, AlpineJS, Vite, Filament, Inertia.js |
| **Ambiente** | MacBook Pro ou Ubuntu, 16-32GB RAM, múltiplos projetos ativos |
| **Dor principal** | Gasta tempo demais gerenciando infraestrutura em vez de escrever código |
| **Comportamento** | Usa terminal fluentemente, mas prefere GUI para tarefas repetitivas |
| **Motivação** | Produtividade máxima, ambiente idêntico ao de produção, zero surpresas no deploy |

### Persona Secundária — "O Tech Lead de Time Pequeno"

| Atributo | Descrição |
|---|---|
| **Perfil** | Líder técnico de time de 2-5 devs |
| **Necessidade** | Padronizar o ambiente de desenvolvimento de toda a equipe via Git |
| **Fluxo** | Commita o arquivo `.orcker.json` no repositório do projeto; todos os membros do time importam e têm o ambiente idêntico em minutos |
| **Valor percebido** | "Funciona na minha máquina" → "Funciona em qualquer máquina do time" |

### Fora do Escopo (Out of Scope)

- Times com infra DevOps dedicada (CI/CD, K8s, infra enterprise)
- Desenvolvedores que usam stacks não-Laravel
- Gerenciamento de ambiente de produção (o produto é exclusivamente para dev local)
- Integração com serviços cloud de deploy (Forge, Vapor, Envoyer) — foco 100% local/dev

---

## 5. Escopo Funcional

A aplicação é organizada em **oito módulos funcionais** e suporta as seguintes stacks de scaffolding: TALL Stack, Filament v3/v4 (Livewire v3/v3.5), Filament v5 (Livewire v4), Inertia.js + Vue 3, Inertia.js + React, API-only e Jetstream.

---

### M1 — Infraestrutura Global (Global Stack)

**Responsabilidade:** Gerenciar os serviços compartilhados que servem a todos os projetos simultaneamente.

#### RF-M1.01 — Configuração da Rede Docker Global
- O sistema deve criar e gerenciar uma rede Docker do tipo `bridge external` nomeada (ex.: `orcker-global`) automaticamente na primeira execução.
- O usuário deve poder renomear a rede global nas configurações.
- O sistema deve verificar a existência da rede antes de cada operação e recriá-la se necessário.

#### RF-M1.02 — Catálogo de Serviços Globais
O sistema deve oferecer um catálogo de serviços prontos para uso como componentes globais:

| Serviço | Imagem Padrão | Porta Padrão | Finalidade |
|---|---|---|---|
| **Redis** | `redis:7-alpine` | 6379 | Cache e filas (Horizon) |
| **PostgreSQL** | `postgres:16-alpine` | 5432 | Banco de dados relacional |
| **MySQL/MariaDB** | `mysql:8` / `mariadb:11` | 3306 | Banco de dados alternativo |
| **Mailpit** | `axllent/mailpit` | 8025/1025 | Captura de e-mails dev |
| **MinIO** | `minio/minio` | 9000/9001 | S3-compatível storage |
| **Soketi** | `quay.io/soketi/soketi` | 6001 | WebSockets (Pusher-compat) |
| **Meilisearch** | `getmeili/meilisearch` | 7700 | Full-text search |
| **Typesense** | `typesense/typesense` | 8108 | Search engine alternativo |

#### RF-M1.03 — Ativação Seletiva de Serviços Globais
- O usuário deve poder ativar/desativar cada serviço global individualmente via toggle.
- O sistema deve persistir a configuração de quais serviços globais estão habilitados.
- O sistema deve exibir o status em tempo real de cada serviço (Running / Stopped / Error / Starting).

#### RF-M1.04 — Configuração por Serviço Global
- Cada serviço global deve ter um painel de configuração acessível com:
  - Versão da imagem (com lista de versões disponíveis via Docker Hub API)
  - Porta exposta (editável, com validação de conflito)
  - Variáveis de ambiente (editor key-value)
  - Volume de persistência (caminho configurável)
  - Recursos (CPU e memória máximos — via `deploy.resources.limits`)

#### RF-M1.05 — Controles Globais Rápidos
- Botão **"Global ON"**: sobe todos os serviços globais ativos em ordem de dependência.
- Botão **"Global OFF"**: para todos os serviços globais.
- Botão **"Restart Global"**: reinicia todos os serviços globais.
- Ação acessível via atalho de teclado configurável.

---

### M2 — Gerenciamento de Projetos

**Responsabilidade:** Cadastro, configuração e ciclo de vida de projetos Laravel.

#### RF-M2.01 — Cadastro de Projeto
O usuário deve poder registrar um projeto com os seguintes atributos:
- Nome do projeto (identificador amigável)
- Caminho absoluto no sistema de arquivos (selecionado via file picker nativo)
- Tipo de banco de dados preferido (PostgreSQL / MySQL / MariaDB / SQLite)
- Porta HTTP local (validação de conflito automática)
- Porta Vite/HMR (padrão: 5173, com detecção de conflito)
- Versão do PHP (selecionável: 8.2, 8.3, 8.4)
- Descrição (opcional)
- Tags/grupos (para organização)

#### RF-M2.02 — Importação de Projeto Existente
- O sistema deve detectar automaticamente projetos Laravel existentes ao apontar para um diretório:
  - Verificar presença de `artisan`, `composer.json`, `.env`
  - Detectar `docker-compose.yml` existente e oferecer migração guiada
  - Detectar versão do Laravel via `composer.json`

#### RF-M2.03 — Scaffold de Novo Projeto
O sistema deve oferecer um wizard para criação de novo projeto Laravel dockerizado com:
- Nome e diretório de destino
- Versão do Laravel (via `composer create-project`)
- Seleção de stack:
  - **TALL Stack** (Livewire 4 + AlpineJS + TailwindCSS + Vite)
  - **Filament v3/v4** (Laravel + Livewire v3/v3.5 + TailwindCSS + AlpineJS — painel admin completo)
  - **Filament v5** (Laravel + Livewire v4 + TailwindCSS + AlpineJS — painel admin com Livewire 4)
  - **Inertia.js + Vue 3** (Laravel + Inertia + Vue 3 + Vite)
  - **Inertia.js + React** (Laravel + Inertia + React + Vite)
  - **API-only** (sem frontend)
  - **Jetstream (TALL)**
- Seleção de serviços específicos do projeto (além dos globais)
- Geração automática de: `Dockerfile.dev`, `docker-compose.yml`, `nginx/`, `php/`, `supervisord.conf`, `.env.example`
- Formato do `docker-compose.yml` gerado: compatível com **Docker Compose plugin atual** e com **formato v2 legado** (`docker-compose` com hífen), detectado automaticamente pela versão instalada no host

#### RF-M2.03.1 — Comportamento do Vite Dev Server por Projeto
- Cada projeto deve ter uma configuração individual para o Vite Dev Server com duas opções:
  - **Automático:** o processo Vite é iniciado dentro do container `app` na porta 5173 automaticamente ao ligar o projeto
  - **Manual:** o Vite só é iniciado via botão dedicado ou Quick Action, sem inicialização automática
- A preferência é persistida por projeto e editável a qualquer momento no painel do projeto
- O processo Vite sempre roda **dentro do container `app`** (porta 5173), alinhado com o setup de referência

#### RF-M2.04 — Serviços por Projeto (Project-Specific Services)
Além dos serviços globais, o usuário deve poder adicionar serviços específicos ao projeto:

| Serviço | Finalidade |
|---|---|
| **PHP-FPM** (obrigatório) | Runtime da aplicação |
| **Nginx** (obrigatório) | Reverse proxy |
| **Worker dedicado** | Queue worker separado (produção-like) |
| **Scheduler** | Cron via container |
| **Reverb** | WebSockets nativo Laravel |
| **Pulse** | Monitoramento de performance |
| **Horizon** | Dashboard de filas |
| **Vite Dev Server** | Hot-reload frontend |
| **Xdebug** | Debugging PHP |
| **Node.js standalone** | Para projetos com SSR (Nuxt, Inertia) |

#### RF-M2.05 — Painel de Projeto
Cada projeto deve ter um painel dedicado exibindo:
- Status consolidado (All OK / Degraded / Stopped / Error)
- Lista de serviços ativos com status individual
- Ações rápidas (Start / Stop / Restart / Rebuild)
- Acesso rápido às URLs locais (HTTP, Mailpit, MinIO, Horizon, Pulse)
- Consumo de recursos (CPU %, Memória)
- Últimas linhas de log (preview)
- Favoritos de comandos do projeto

#### RF-M2.06 — Variáveis de Ambiente por Projeto
- Editor visual de `.env` com validação de tipo e formato.
- Suporte a múltiplos ambientes: `.env.local`, `.env.testing`.
- Preenchimento automático de variáveis de serviços conectados (ex.: `DB_HOST`, `REDIS_HOST`, `MAIL_HOST`).
- Diff visual entre `.env` atual e `.env.example`.

#### RF-M2.07 — Editor Visual de php.ini
- O Orcker deve oferecer um editor visual de `php.ini` com as diretivas organizadas por categoria (Performance, Upload, Execução, OPcache, Extensões).
- Cada diretiva deve exibir: nome, valor atual (editável), valor padrão do PHP, descrição inline em português e link para a documentação oficial.
- Validação de tipos e intervalos de valor em tempo real (ex.: `memory_limit` aceita sufixo `M`/`G`).
- Modo raw de edição do arquivo completo também disponível para usuários avançados.
- Alterações são aplicadas com reinício automático do container PHP-FPM.

#### RF-M2.08 — Gerenciamento Visual do Supervisor
- O Orcker deve oferecer um painel visual de gerenciamento dos processos do Supervisor por projeto.
- Funcionalidades disponíveis:
  - Adicionar novo processo (nome, comando, diretório, usuário, prioridade, autostart, autorestart)
  - Remover processo existente (com confirmação)
  - Editar configuração de processo existente (número de instâncias `numprocs`, prioridade, `stopwaitsecs`)
  - Start / Stop / Restart de processo individual
  - Exibição de status em tempo real de cada processo (Running / Stopped / Starting / Fatal)
  - Log de output de cada processo acessível com um clique
- A configuração do Supervisor é salva no arquivo `supervisord.conf` do projeto e versionada via Git.

#### RF-M2.09 — Suporte a Xdebug com Integração de IDE
O suporte a Xdebug deve cobrir:
- **Toggle visual** de `XDEBUG_MODE` por projeto (off / debug / coverage / profile) sem edição manual do `.env`
- **Integração com VS Code:** geração automática do arquivo `.vscode/launch.json` com a configuração correta de `Listen for Xdebug` apontando para a porta e path mappings do container
- **Integração com PhpStorm:** geração automática da configuração de PHP Remote Debug no arquivo `.idea/` via configuração de CLI Interpreter e server path mappings
- Detecção automática do IDE presente no sistema (via presença de `.vscode/` ou `.idea/` no diretório do projeto)
- Aviso visual quando Xdebug está ativo, pois impacta performance do PHP-FPM

---

### M3 — Terminal e Comandos Rápidos

**Responsabilidade:** Centralizar a execução de comandos frequentes com atalhos e favoritos.

#### RF-M3.01 — Terminal Integrado
- Terminal embutido na aplicação com suporte a `docker compose exec`.
- Contexto de projeto: ao abrir o terminal em um projeto, o contexto já é o container `app`.
- Histórico de comandos persistido por projeto.
- Múltiplas abas de terminal simultâneas.

#### RF-M3.02 — Paleta de Comandos (Command Palette)
Inspirado no VS Code / Raycast: ativável via `Cmd+K` (macOS) / `Ctrl+K` (Linux/Windows).
- Busca fuzzy em tempo real por comandos Artisan, NPM, Composer, Pest.
- Execução direta no container do projeto ativo.
- Histórico de comandos recentes no topo.

#### RF-M3.03 — Comandos Pré-definidos (Quick Actions)
O sistema deve oferecer um conjunto de ações pré-mapeadas, organizadas por categoria:

**Artisan — Database:**
- `migrate` / `migrate:fresh` / `migrate:fresh --seed` / `migrate:rollback` / `db:seed`

**Artisan — Cache:**
- `cache:clear` / `config:clear` / `route:clear` / `view:clear` / `optimize:clear`

**Artisan — Geração de Código:**
- `make:model` / `make:controller` / `make:migration` / `make:job` / `make:event` / `make:listener` / `make:policy` / `make:seeder` / `make:factory` / `make:test`

**Artisan — Utilitários:**
- `tinker` / `queue:work` / `schedule:run` / `key:generate` / `storage:link`

**Composer:**
- `install` / `update` / `dump-autoload` / `require [package]`

**NPM/Vite:**
- `npm install` / `npm run dev` / `npm run build`

**Testes:**
- `php artisan test` / `./vendor/bin/pest` / `./vendor/bin/pest --coverage` / `./vendor/bin/pest --filter [name]`

**Code Quality:**
- `./vendor/bin/pint` / `./vendor/bin/pint --test`

#### RF-M3.04 — Favoritos de Comandos
- O usuário deve poder marcar qualquer comando como favorito por projeto.
- Favoritos ficam acessíveis via painel lateral e via atalho numérico (`Cmd+1`, `Cmd+2`...).
- Suporte a comandos personalizados com parâmetros fixos ou variáveis (prompts inline).
- Organização de favoritos em grupos nomeados.

#### RF-M3.05 — Histórico de Execução
- Registro de todos os comandos executados com: timestamp, projeto, comando completo, duração, status (success/error), output (truncado).
- Busca e filtragem no histórico.
- Re-execução de comando do histórico com um clique.

---

### M4 — Logs e Observabilidade

**Responsabilidade:** Visibilidade centralizada do estado dos serviços e da aplicação.

#### RF-M4.01 — Visualizador de Logs Centralizado
- Painel de logs unificado com selector de fonte:
  - Logs de containers Docker (`docker logs -f [container]`)
  - `storage/logs/laravel.log` (tail em tempo real)
  - Logs do Nginx
  - Logs do Supervisor
- Colorização de níveis de log (DEBUG, INFO, WARNING, ERROR, CRITICAL).
- Filtro por nível, keyword e intervalo de tempo.
- Busca com highlight e navegação entre ocorrências.
- Export de logs para arquivo.

#### RF-M4.02 — Dashboard de Recursos
- Visão global de todos os containers ativos com métricas em tempo real:
  - CPU % por container
  - Memória usada / limite por container
  - I/O de rede (bytes in/out)
  - I/O de disco
- Gráfico de série temporal (últimos 5 minutos) para CPU e memória.
- Alertas visuais quando container ultrapassa thresholds configuráveis.

#### RF-M4.03 — Notificações de Status
- Notificações do sistema (OS notification) para eventos:
  - Container em estado `Error` ou `Exited` inesperadamente
  - Build concluído com sucesso ou falha
  - Comando de longa duração finalizado
  - Novo output de erro nos logs Laravel
- Sistema de notificações interno (sino na UI) com histórico.

#### RF-M4.04 — Health Monitor
- Exibição dos healthchecks configurados nos containers (ex.: PostgreSQL `pg_isready`).
- Status visual por container: Healthy / Unhealthy / Starting / No healthcheck.
- Timeline de quando o container ficou saudável após startup.

---

### M5 — Configuração de Infraestrutura

**Responsabilidade:** Geração, edição e versionamento dos arquivos de infraestrutura.

#### RF-M5.01 — Editor de docker-compose.yml
- Editor visual com abas para cada serviço.
- Modo raw YAML com syntax highlight e validação em tempo real.
- Preview das mudanças antes de aplicar (`docker compose config`).
- Detecção de conflitos de porta entre projetos.
- **Suporte a Docker Compose v2 legado** (`docker-compose.yml` com `version: "2.x"/"3.x"`): o Orcker detecta o formato ao importar um projeto existente, oferece migração guiada para o formato atual do plugin, ou mantém compatibilidade com o binário `docker-compose` (com hífen) caso seja o binário presente no host.

#### RF-M5.02 — Gerenciamento de Volumes
- Listagem de todos os volumes Docker (globais e por projeto).
- Informação de tamanho, último acesso e projeto associado.
- Ação de limpeza de volumes órfãos (`docker volume prune`).
- Backup e restore de volumes de banco de dados (via `pg_dump` / `mysqldump` integrado).

#### RF-M5.03 — Gerenciamento de Imagens
- Listagem de imagens locais com tamanho, data de criação e uso.
- Rebuild de imagem individual ou completo.
- Limpeza de imagens não utilizadas (`docker image prune`).
- Pull manual de atualizações de imagens.

#### RF-M5.04 — Templates de Infraestrutura
O sistema deve oferecer templates pré-configurados baseados no documento de referência:

| Template | Stack base | Observação |
|---|---|---|
| **TALL Stack Completo** | PHP-FPM + Nginx + Vite + Supervisor (Horizon + Schedule + Pulse) | Livewire 4 |
| **Filament v3/v4** | PHP-FPM + Nginx + Supervisor + TailwindCSS CLI | Livewire v3/v3.5 — sem Vite como servidor HMR; assets compilados via Node.js no build |
| **Filament v5** | PHP-FPM + Nginx + Vite + Supervisor | Livewire v4 — hot-reload via Vite nativo |
| **Inertia.js + Vue 3** | PHP-FPM + Nginx + Vite + Node.js + Supervisor | — |
| **Inertia.js + React** | PHP-FPM + Nginx + Vite + Node.js + Supervisor | — |
| **API Laravel** | PHP-FPM + Nginx + Queue Worker (sem Vite) | — |
| **Microserviço** | PHP-FPM mínimo, sem Nginx (comunicação interna) | — |

**Nota técnica — Filament e Vite Dev Server:**
- **Filament v3/v4** utiliza Livewire v3/v3.5, que não depende do servidor Vite para hot-reload em desenvolvimento. O Orcker deve desabilitar automaticamente o RF-M2.03.1 (Vite Dev Server configurável) para projetos Filament v3/v4 e substituí-lo por um processo de build de assets via `npm run build` sob demanda. Assets do painel são servidos via `php artisan filament:assets`.
- **Filament v5** utiliza Livewire v4, que tem integração nativa com Vite. O Orcker habilita o Vite Dev Server normalmente (RF-M2.03.1), com hot-reload funcional para componentes Filament.

Templates são editáveis pelo usuário e podem ser salvos como templates customizados. Templates da comunidade são distribuídos via marketplace (ver RF-M5.06).

#### RF-M5.05 — Versionamento de Configuração
- Todo arquivo de infraestrutura gerado pelo Orcker deve ser versionado em Git automaticamente (se o projeto tiver um repositório Git).
- O arquivo `.orcker.json` (configuração do projeto) deve ser versionado junto ao repositório, permitindo que todos os membros do time importem o ambiente com um clique.
- Histórico de alterações com diff visual.
- Capacidade de reverter para versão anterior de qualquer arquivo de infra.
- **Sincronização ao abrir:** ao iniciar o Orcker com um projeto já configurado, o sistema compara o estado atual dos arquivos de infra com o estado gerenciado e exibe um aviso visual se houver divergências — sem bloquear o fluxo nem sobrescrever alterações externas automaticamente.

#### RF-M5.06 — Marketplace de Templates da Comunidade
- O Orcker deve integrar um marketplace de templates de infraestrutura contribuídos pela comunidade (ex.: Laravel Octane + Swoole, Laravel Reverb + Soketi, setups especializados).
- Acesso ao marketplace diretamente na tela de criação de novo projeto.
- Templates indexados em repositório público no GitHub do projeto Orcker.
- Qualquer desenvolvedor pode submeter um template via Pull Request seguindo o schema definido.
- Avaliação, número de downloads e data de última atualização exibidos por template.

---

### M6 — Gerenciamento de Banco de Dados

**Responsabilidade:** Operações de banco de dados sem sair da ferramenta.

#### RF-M6.01 — Conexões de Banco
- O sistema deve detectar automaticamente as conexões de banco configuradas no `.env` de cada projeto.
- Listagem de bancos disponíveis no serviço global PostgreSQL/MySQL.
- **Criação automática do banco de testes:** ao registrar ou fazer scaffold de um novo projeto, o Orcker deve criar automaticamente um banco de dados de testes nomeado `{projeto}_testing` no serviço global, replicando o comportamento do `init.sql` de referência. O banco de testes fica pronto para uso imediato com `php artisan test` sem nenhuma configuração adicional pelo dev.
- A criação automática pode ser desativada nas configurações do projeto para casos que exigem configuração manual.

#### RF-M6.02 — Operações Rápidas de Banco
- Interface para execução rápida de:
  - Migrate / Fresh / Seed (com confirmação para `fresh`)
  - Dump de banco (export SQL com timestamp)
  - Restore de dump
  - Conexão ao PSQL/MySQL CLI diretamente no terminal integrado

#### RF-M6.03 — Visualizador de Queries (Opcional — Roadmap)
- Integração com Laravel Telescope para exibição de queries recentes.
- Alertas de slow queries configuráveis por threshold.

---

### M7 — Configurações e Preferências

**Responsabilidade:** Personalização da aplicação e do ambiente global.

#### RF-M7.01 — Configurações Globais da Aplicação
- Tema: Dark / Light / System
- Idioma: Português / English
- Fonte do editor de código (para terminal e editores internos)
- Tamanho de fonte
- Atalhos de teclado (totalmente remapeáveis)
- Comportamento na inicialização do SO (abrir automaticamente, comportamento da system tray)

#### RF-M7.02 — Configurações de Docker Global
- Caminho do socket Docker (auto-detectado; configurável para casos especiais como Colima/Orbstack no macOS)
- Limite de recursos padrão para novos containers (CPU, Memória)
- Política de restart padrão por ambiente (`no` para dev)
- Registro de imagens alternativo (para empresas com registry privado)

#### RF-M7.03 — Sistema Tray
- Ícone na system tray com indicador de status geral (verde/amarelo/vermelho).
- Menu rápido de acesso via clique no ícone:
  - Start All / Stop All
  - Lista dos projetos recentes com status
  - Abrir janela principal
  - Quit

#### RF-M7.04 — Backup e Restore de Configurações do Orcker
- Export de toda a configuração do Orcker (projetos, serviços, favoritos) em arquivo JSON portável.
- Import de configuração em nova máquina.
- Suporte a sincronização via arquivo (para manter máquinas sincronizadas manualmente).

---

### M8 — Sandbox & Compartilhamento

**Responsabilidade:** Expor um projeto Laravel local à internet de forma segura e temporária para demonstrações, homologações e testes com clientes ou empregadores, sem necessidade de deploy em ambiente remoto.

#### RF-M8.01 — Ativação do Sandbox por Projeto
- O Orcker deve oferecer, em cada projeto, uma ação **"Iniciar Sandbox"** que sobe um processo de túnel reverso apontando para a porta HTTP local do projeto (padrão `:80`).
- O sandbox está **desabilitado por padrão** — deve ser ativado explicitamente pelo dev a cada sessão. Nenhuma sessão persiste entre reinicializações do Orcker.
- Ao ativar, o Orcker exibe um aviso de segurança claro e proeminente informando que o projeto ficará acessível externamente, com resumo dos controles de proteção ativos.
- O processo de túnel é gerenciado pelo Orcker como qualquer outro serviço: start, stop, status em tempo real, logs de processo.
- Um indicador visual permanente (badge pulsante) é exibido na sidebar do projeto e na system tray enquanto qualquer sandbox estiver ativo.
- O Orcker encerra automaticamente todos os sandboxes ativos ao ser fechado.

#### RF-M8.02 — URL Pública e QR Code
- Ao ativar o sandbox, o Orcker captura e exibe a URL pública HTTPS gerada pelo provedor (ex.: `https://saas-app-k9x2m.orcker.dev`).
- A URL é copiável com um clique.
- Um QR Code é gerado localmente (sem chamada de API externa) e exibido no painel, permitindo que o cliente abra o projeto apontando a câmera do celular.
- A URL e o QR são regenerados a cada nova sessão de sandbox para evitar reutilização não autorizada.

#### RF-M8.03 — Provedores de Túnel (multi-provedor)
O Orcker deve suportar três provedores, selecionáveis por projeto nas configurações do sandbox:

| Provedor | Custo | Conta obrigatória | Diferencial |
|---|---|---|---|
| **Cloudflare Tunnel** | Gratuito | Não (modo rápido) | Zero configuração; HTTPS automático; binário `cloudflared` bundled no instalador |
| **ngrok** | Free tier / Pago | Sim (API token) | URLs persistentes e nomeadas; painel de inspeção de requisições HTTP |
| **Expose (BeyondCode)** | Self-hosted | Self-hosted | Laravel-native; pode rodar no próprio servidor do desenvolvedor |

- O binário `cloudflared` é **bundled** no instalador do Orcker para todas as plataformas — nenhuma dependência externa para o provedor padrão.
- Tokens de autenticação (ngrok, Expose) são armazenados exclusivamente no keychain nativo do SO, nunca em arquivos de configuração em texto plano.

#### RF-M8.04 — Controles de Segurança da Sessão
- **Proteção por senha (opt-in, recomendada):** middleware injetado no proxy do túnel exige uma senha antes de renderizar qualquer rota do projeto. A senha é gerada automaticamente (forte, 12 caracteres) podendo ser substituída pelo dev. Exibida com opção de copiar e ocultar.
- **Expiração automática (obrigatória):** o sandbox encerra automaticamente após o tempo configurado. Opções: 1h, 2h, 4h, 8h, 24h. Padrão: 4h. Não há opção "sem expiração" — toda sessão tem um fim definido.
- **Allowlist de IPs (opt-in):** restringe o acesso a faixas de IP específicas (ex.: IP fixo do cliente). Campo de entrada com validação de CIDR.
- **Modo somente leitura (opt-in):** bloqueia requisições `POST`, `PUT`, `PATCH`, `DELETE` no nível do proxy, permitindo apenas navegação e leitura. Útil para demonstrações onde o cliente não deve alterar dados.
- Renovação de sessão disponível antes do vencimento, sem necessidade de gerar nova URL (compatível com Cloudflare Tunnel).

#### RF-M8.05 — Log de Acesso em Tempo Real
- Painel de log exibindo em tempo real por requisição: bandeira do país de origem (geolocalização por IP via banco local GeoIP — sem chamada de API externa), IP mascarado (últimos dois octetos ocultos), rota e método HTTP, status code e timestamp relativo.
- Contadores de sessão: total de requisições, visitantes únicos (por IP), latência média, pico de latência.
- Highlight visual de requisições com status de erro (4xx, 5xx).
- Export do log completo em CSV ao encerrar ou durante a sessão ativa.

#### RF-M8.06 — Subdomínio Personalizado com orcker.dev *(Fase 3)*
- Integração com DNS da `orcker.dev` para oferecer subdomínios fixos e nomeados por projeto (ex.: `meu-saas.orcker.dev`) como feature da comunidade open-source.
- Requer conta gratuita registrada em `orcker.dev` e tunnel permanente configurado via Cloudflare.
- Subdomínio persiste entre sessões, permitindo que o cliente acesse sempre o mesmo endereço durante um ciclo de homologação.

---

## 6. Requisitos Não-Funcionais

### RNF-01 — Performance
- Tempo de inicialização da aplicação: < 2 segundos (cold start).
- Tempo de resposta da UI para ações do usuário: < 100ms (feedback imediato).
- Atualização de métricas em tempo real: polling a cada 2 segundos (configurável).
- Consumo de RAM pela aplicação: < 150MB em idle, < 300MB em uso intenso.
- Consumo de CPU pela aplicação: < 2% em idle.

### RNF-02 — Compatibilidade de Plataforma
| OS | Versão Mínima | Arquitetura — MVP | Arquitetura — Roadmap |
|---|---|---|---|
| **macOS** | 13 (Ventura) | Intel x64 + Apple Silicon arm64 | — |
| **Linux** | Ubuntu 22.04+ / Debian 11+ / Fedora 38+ | x64 apenas | arm64 (Fase 3+) |
| **Windows** | 10 22H2 / 11 | x64 | — |

- No macOS: suporte a Docker Desktop, OrbStack e Colima.
- No Linux: suporte ao Docker Engine instalado diretamente. Suporte a Docker Compose plugin (`docker compose`) e binário legado (`docker-compose`) detectado automaticamente.
- No Windows: suporte a Docker Desktop com WSL2 backend.
- Linux ARM64 (Raspberry Pi, instâncias ARM) fora do escopo do MVP — planejado para Fase 3.

### RNF-03 — Segurança
- A aplicação nunca deve armazenar credenciais (senhas, API keys) em texto plano.
- Segredos devem ser armazenados no keychain nativo do SO (macOS Keychain, Windows Credential Manager, Linux Secret Service/libsecret).
- A comunicação com o Docker Engine deve ser feita via socket local — nunca via TCP exposto sem TLS.
- Permissões mínimas: a aplicação não deve solicitar execução como `root`.
- Validação de todos os inputs do usuário antes de passá-los como argumentos a comandos do sistema (prevenção de command injection).
- Logs internos da aplicação não devem conter credenciais ou dados sensíveis.
- **Sandbox (M8) — regras adicionais:**
  - O módulo Sandbox nunca deve estar ativo ao iniciar o Orcker — exige ativação explícita por sessão a cada execução.
  - Toda sessão de sandbox deve ter expiração obrigatória — a opção "sem limite de tempo" não existe.
  - Nenhum dado de credencial do projeto (`.env`, senhas de banco) deve trafegar na URL pública sem autenticação habilitada na sessão.
  - O binário `cloudflared` bundled deve ser verificado por hash SHA-256 a cada atualização do Orcker antes de ser executado.
  - Ao encerrar o Orcker (fechar a janela ou via system tray), todos os processos de túnel ativos devem ser terminados imediatamente.

### RNF-04 — Confiabilidade
- A aplicação deve ser estável mesmo quando o Docker Engine não está disponível (UI degradada, não crash).
- Operações de longa duração (build, migrate) não devem bloquear a UI.
- Toda operação destrutiva (fresh, volume delete, image prune) deve exigir confirmação explícita.
- O sistema deve se recuperar graciosamente de containers em estado `Error`.

### RNF-05 — Manutenibilidade
- Cobertura de testes: ≥ 80% do código de lógica de negócio.
- Separação clara entre camada de UI e camada de lógica (arquitetura em camadas).
- Documentação inline de todas as funções públicas.
- CI/CD automatizado (GitHub Actions) para build e release multiplataforma.

### RNF-06 — Instalação e Distribuição
- **Licença:** MIT — código 100% aberto, repositório público no GitHub desde o MVP.
- Instaladores nativos por plataforma gerados automaticamente via `tauri-action` no GitHub Actions:
  - macOS: `.dmg` (universal binary — Intel + Apple Silicon)
  - Linux: `.AppImage` (portável), `.deb` e `.rpm`
  - Windows: `.msi` / `.exe` com NSIS
- Auto-update integrado via **Tauri Updater** (gratuito para projetos open-source) — notificação de nova versão sem prompt intrusivo; o usuário decide quando atualizar.
- Releases publicados diretamente no GitHub Releases com changelogs gerados automaticamente.
- Instalação sem dependências externas (o Docker Engine é pré-requisito, mas é verificado na primeira execução com guia de instalação por OS caso ausente).

### RNF-07 — Acessibilidade
- Suporte completo a navegação por teclado.
- Contraste mínimo WCAG AA em todos os elementos interativos.
- Atalhos de teclado documentados e acessíveis.

---

## 7. Arquitetura de Alto Nível

### 7.1 Visão Geral

```
┌──────────────────────────────────────────────────────────────────────┐
│                         ORCKER APPLICATION                          │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    UI LAYER (Vue 3 + Tauri)                  │    │
│  │  Components │ Views │ Composables │ Pinia Stores │ Router    │    │
│  └────────────────────────┬────────────────────────────────────┘    │
│                           │ Tauri IPC (invoke/emit)                  │
│  ┌────────────────────────▼────────────────────────────────────┐    │
│  │                  CORE LAYER (Rust / Tauri)                   │    │
│  │  Commands │ Events │ State │ Config │ Keychain │ FS          │    │
│  └────────────────────────┬────────────────────────────────────┘    │
│                           │                                          │
│  ┌────────────────────────▼────────────────────────────────────┐    │
│  │              INFRASTRUCTURE ADAPTERS (Rust)                  │    │
│  │  DockerAdapter │ GitAdapter │ FileSystemAdapter │ ShellAdapt │    │
│  └────────────────────────┬────────────────────────────────────┘    │
│                           │                                          │
└───────────────────────────┼──────────────────────────────────────────┘
                            │
         ┌──────────────────┼───────────────────┐
         │                  │                    │
         ▼                  ▼                    ▼
  Docker Engine API    File System          Git CLI / API
  (unix socket /       (projetos,           (versionamento
   named pipe)         configs, logs)        de configs)
```

### 7.2 Padrão Arquitetural

O Orcker adota uma arquitetura em camadas com separação clara de responsabilidades:

**UI Layer (Frontend — Vue 3)**
- Responsável exclusivamente por renderização e interação com o usuário.
- Não realiza nenhuma operação de sistema diretamente.
- Comunica-se com o Core via Tauri IPC (`invoke` para comandos, `listen` para eventos).

**Core Layer (Rust — Tauri Commands)**
- Contém toda a lógica de negócio da aplicação.
- Gerencia o estado da aplicação (projetos, serviços, configurações).
- Orquestra chamadas para os adapters de infraestrutura.
- Emite eventos para a UI sobre mudanças de estado.

**Infrastructure Adapters (Rust)**
- **DockerAdapter:** Comunicação com Docker Engine via API REST sobre socket Unix/named pipe.
  - Endpoints utilizados: containers, images, volumes, networks, exec, stats, logs.
- **FileSystemAdapter:** Leitura e escrita de arquivos de configuração (docker-compose, Dockerfile, .env, nginx.conf).
- **GitAdapter:** Operações Git via `git2` (Rust crate) para versionamento de configs.
- **ShellAdapter:** Execução de comandos arbitrários no container via `docker exec`.
- **KeychainAdapter:** Integração com keychain do SO para armazenamento seguro de credenciais.

### 7.3 Estratégia de Estado

O estado da aplicação é dividido em:

| Tipo de Estado | Armazenamento | Tecnologia |
|---|---|---|
| Configuração persistente | Arquivo JSON no diretório de dados do app | Tauri Store / serde_json |
| Credenciais/Segredos | Keychain nativo do SO | keyring-rs |
| Estado em memória (containers, métricas) | Pinia (Vue 3) | Vue Reactivity |
| Logs em tempo real | Buffer em memória + stream via IPC | Tauri Events |

### 7.4 Comunicação com Docker

O Orcker comunica-se com o Docker Engine exclusivamente via **Docker Engine API v1.44+** sobre:
- **Linux/macOS:** Unix socket em `/var/run/docker.sock` (ou caminho configurável para OrbStack/Colima)
- **Windows:** Named pipe em `//./pipe/docker_engine`

Nenhuma chamada ao CLI do `docker` ou `docker compose` é feita diretamente — toda interação é via API, garantindo controle total sobre erros e outputs.

**Exceção:** Para operações de `docker compose up/down`, o sistema pode optar por gerar o `docker-compose.yml` e invocar o CLI de forma controlada, capturando stdout/stderr em tempo real.

---

## 8. UX/UI — Diretrizes e Princípios

### 8.1 Filosofia de Design

**Dark-First, Clean, Tecnológico.**
A interface deve transmitir **competência e controle** — como a tela de um cockpit de aviação ou um painel de monitoramento profissional. Tecnológica sem ser intimidadora. Densa na informação onde necessário, mas com hierarquia visual clara.

> "Um painel de controle que um desenvolvedor se orgulhe de ter aberto."

### 8.2 Design System

**Paleta de Cores (Dark Mode — Default)**

| Token | Valor | Uso |
|---|---|---|
| `--bg-base` | `#0D0E14` | Background principal |
| `--bg-surface` | `#13151F` | Cards e painéis |
| `--bg-elevated` | `#1A1D2E` | Elementos elevados, modals |
| `--bg-border` | `#252840` | Bordas e divisores |
| `--accent-primary` | `#6366F1` | Ações primárias (Indigo) |
| `--accent-secondary` | `#22D3EE` | Destaque secundário (Cyan) |
| `--status-success` | `#22C55E` | Running, OK |
| `--status-warning` | `#EAB308` | Starting, Degraded |
| `--status-error` | `#EF4444` | Error, Exited |
| `--status-inactive` | `#6B7280` | Stopped, Inactive |
| `--text-primary` | `#F1F5F9` | Texto principal |
| `--text-secondary` | `#94A3B8` | Texto secundário |
| `--text-muted` | `#475569` | Texto de dica, label |

**Tipografia**
- Família: `JetBrains Mono` (para elementos de código/terminal) + `Inter` ou `Geist` (para UI geral)
- Tamanho base: 14px
- Escala modular: 12 / 14 / 16 / 20 / 24 / 32px

**Iconografia**
- Biblioteca: Lucide Icons (SVG, consistente, open-source)
- Tamanho padrão: 16px (inline), 20px (ações), 24px (destaque)

### 8.3 Layout Principal

```
┌────────────────────────────────────────────────────────────────────┐
│  SIDEBAR (240px)         │  MAIN CONTENT AREA                      │
│  ┌──────────────────┐   │  ┌──────────────────────────────────┐   │
│  │ 🔷 Orcker        │   │  │  Header da View Atual            │   │
│  ├──────────────────┤   │  ├──────────────────────────────────┤   │
│  │ 🌐 Global Stack  │   │  │                                  │   │
│  │ ─────────────    │   │  │  Conteúdo principal              │   │
│  │ PROJETOS         │   │  │                                  │   │
│  │ > Projeto A ●   │   │  │                                  │   │
│  │   Projeto B ○   │   │  │                                  │   │
│  │   Projeto C ●   │   │  │                                  │   │
│  │ + Novo Projeto  │   │  │                                  │   │
│  │ ─────────────    │   │  │                                  │   │
│  │ ⚙ Configurações │   │  └──────────────────────────────────┘   │
│  │ 📋 Histórico     │   │                                         │
│  │ 📊 Recursos      │   │  BOTTOM PANEL (toggle)                  │
│  └──────────────────┘   │  ┌──────────────────────────────────┐   │
│                          │  │  Terminal / Logs / Output         │   │
│                          │  └──────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────  ┘
│ STATUS BAR: Docker Connected ● | 3 containers running | CPU 12%    │
└────────────────────────────────────────────────────────────────────┘
```

### 8.4 Princípios de UX

1. **Progressive Disclosure:** Informações básicas visíveis por padrão; detalhes acessíveis sob demanda (expand/collapse, tooltips, modals).
2. **Ação com Confirmação para Operações Destrutivas:** `migrate:fresh`, `volume delete`, `image prune` sempre pedem confirmação explícita com descrição clara do impacto.
3. **Feedback Imediato:** Toda ação deve gerar feedback visual em < 100ms (spinner, progress, toast).
4. **Estado Sempre Visível:** O estado de cada container deve estar sempre visível na sidebar sem precisar navegar.
5. **Keyboard-First:** Todo fluxo principal deve ser completável via teclado. Command Palette como ponto central de acesso rápido.
6. **Zero Configuração Necessária para Começar:** Defaults sensatos em tudo. O usuário deve conseguir usar 80% das funcionalidades sem tocar nas configurações.

### 8.5 Padrões de Feedback de Estado

| Estado | Indicador Visual | Cor | Ação Disponível |
|---|---|---|---|
| Running | Ponto pulsante | Verde | Stop, Restart, Logs |
| Stopped | Ponto estático | Cinza | Start |
| Starting | Anel girando | Amarelo | Logs |
| Error | X pulsante | Vermelho | Logs, Restart, Details |
| Building | Barra de progresso | Azul | Cancel |
| Healthy | Check mark | Verde | — |
| Unhealthy | Triângulo | Amarelo | Logs, Restart |

---

## 9. Modelo de Dados Lógico

### 9.1 Entidades Principais

```
GlobalConfig
├── dockerSocketPath: string
├── globalNetworkName: string
├── defaultResourceLimits: ResourceLimits
└── globalServices: GlobalService[]

GlobalService
├── id: UUID
├── type: ServiceType (redis|postgres|mysql|mailpit|minio|soketi|...)
├── enabled: boolean
├── imageTag: string
├── exposedPort: number
├── envVars: EnvVar[]
├── volumePath: string
└── resourceLimits: ResourceLimits

Project
├── id: UUID
├── name: string
├── path: string (absolute)
├── description: string?
├── tags: string[]
├── phpVersion: string
├── databaseType: DatabaseType
├── httpPort: number
├── vitePort: number
├── services: ProjectService[]
├── favoriteCommands: FavoriteCommand[]
├── createdAt: DateTime
└── updatedAt: DateTime

ProjectService
├── id: UUID
├── type: ServiceType
├── enabled: boolean
├── config: ServiceConfig (union type por ServiceType)
└── dependsOn: ServiceType[]

FavoriteCommand
├── id: UUID
├── label: string
├── command: string (template com variáveis)
├── context: ServiceType (app|db|redis|...)
├── group: string?
└── shortcut: string? (ex: "cmd+1")

CommandHistoryEntry
├── id: UUID
├── projectId: UUID
├── command: string
├── context: string
├── exitCode: number
├── duration: number (ms)
├── executedAt: DateTime
└── outputPreview: string (primeiros 500 chars)

SandboxSession
├── id: UUID
├── projectId: UUID
├── provider: TunnelProvider (cloudflare|ngrok|expose)
├── publicUrl: string
├── passwordHash: string? (bcrypt — null se desabilitado)
├── readOnly: boolean
├── ipAllowlist: string[] (CIDRs — vazio = sem restrição)
├── expiresAt: DateTime (obrigatório — sem sessão perpétua)
├── startedAt: DateTime
├── endedAt: DateTime?
├── requestCount: number
├── uniqueVisitors: number
└── accessLog: SandboxAccessLogEntry[]

SandboxAccessLogEntry
├── id: UUID
├── sessionId: UUID
├── timestamp: DateTime
├── ipMasked: string (dois últimos octetos ocultos)
├── countryCode: string (ISO 3166-1 alpha-2 via GeoIP local)
├── method: HttpMethod
├── path: string
└── statusCode: number
```

---

## 10. Stack Recomendada

### 10.1 Análise e Justificativa

Para um desenvolvedor solo com perfil Laravel/TALL Stack, a escolha da stack de desenvolvimento do Orcker deve equilibrar:
- **Proximidade com o conhecimento existente** (reduz curva de aprendizado)
- **Adequação técnica** para aplicação desktop multiplataforma
- **Maturidade do ecossistema** (documentação, comunidade, longevidade)
- **Velocidade de desenvolvimento** (solo dev, sem equipe de suporte)

### 10.2 Stack Recomendada — Tauri 2 + Vue 3 + TypeScript

#### Framework Desktop: **Tauri 2**

| Critério | Tauri 2 | Electron | Por que Tauri vence |
|---|---|---|---|
| Tamanho do binário | ~8-15MB | ~150-200MB | Crítico para UX de instalação |
| Consumo de RAM | ~50-80MB | ~200-400MB | Respeita RNF-01 |
| Segurança | Excelente (Rust + CSP rígido) | Moderada (Chromium + Node.js) | Respeita RNF-03 |
| Performance | Nativo (Rust backend) | JavaScript single-threaded | Melhor para operações de I/O intensivo |
| Multiplataforma | Excelente (Windows/macOS/Linux) | Excelente | Empate |
| Curva de aprendizado | Rust (backend) — moderada | Node.js — baixa | Electron tem vantagem aqui |
| Auto-updater | Embutido (Tauri Updater) | Electron-updater | Empate |

**Mitigação da curva Rust:** A camada Rust do Tauri para este projeto é predominantemente "boilerplate" + chamadas de API. A complexidade de Rust avançado (lifetimes, ownership em concorrência) é encapsulada pelo próprio Tauri. O desenvolvedor escreverá 80% do código em Vue/TypeScript (familiar) e ~20% em Rust (aprendizado incremental).

#### UI Framework: **Vue 3 + TypeScript**

Justificativa para o perfil do desenvolvedor:

| Analogia TALL Stack → Vue 3 |
|---|
| **Livewire components** → **Vue Single File Components (.vue)** |
| **AlpineJS `x-data`** → **Vue `ref()` / `reactive()`** |
| **AlpineJS `@click`** → **Vue `@click`** |
| **Livewire actions** → **Vue methods / composables** |
| **Blade templates** → **Vue `<template>`** |

A transição mental de TALL para Vue 3 é a mais suave disponível no ecossistema JS. O desenvolvedor já pensa em componentes, reatividade e eventos — conceitos idênticos.

#### Gerenciamento de Estado: **Pinia**

Store oficial do Vue 3. Simples, tipado, sem boilerplate excessivo. Análogo ao contexto de estado que o desenvolvedor já usa no PHP (Service Providers, Singletons).

#### Roteamento: **Vue Router 4**

Roteamento client-side para as views da aplicação.

#### Estilização: **TailwindCSS 3**

O desenvolvedor já domina TailwindCSS 4. Usar TailwindCSS no Orcker significa que **nenhum CSS novo precisa ser aprendido**. A curva de aprendizado é zero nessa camada.

> **Nota:** Preferir TailwindCSS 3 no momento pela maior maturidade em ecossistemas Tauri. TailwindCSS 4 pode ser adotado quando a integração estiver estável.

#### Componentes UI: **Radix Vue + shadcn/vue**

Componentes acessíveis, sem estilo opinionado (estilizados com Tailwind). Oferece: Dialog, Tooltip, Dropdown, Toast, Tabs, Select — todos os primitivos necessários para a UI do Orcker.

#### Comunicação com Docker: **bollard (Rust crate)**

Biblioteca Rust para comunicação com Docker Engine API via socket. Madura, bem mantida, usada em produção por ferramentas como Podman Desktop.

#### Persistência de Configuração: **tauri-plugin-store**

Plugin oficial do Tauri para persistência de dados estruturados (JSON) no diretório de dados do app (segue as convenções de cada OS).

#### Keychain/Segredos: **keyring-rs**

Crate Rust para integração com keychains nativos de cada OS (Keychain no macOS, Credential Manager no Windows, Secret Service no Linux).

#### Terminal Emulado: **xterm.js**

Emulador de terminal em JavaScript, usado pelo VS Code e outros projetos de grande escala. Integração com Tauri via IPC para streaming de output de comandos.

#### Testes — Rust: **cargo test + tokio-test**

Para lógica do backend em Rust.

#### Testes — Vue: **Vitest + Vue Test Utils**

Para componentes e composables Vue.

### 10.3 Resumo da Stack Final

```
┌─────────────────────────────────────────────────────────────┐
│                    ORCKER — TECH STACK                      │
├─────────────────────┬───────────────────────────────────────┤
│ Desktop Framework   │ Tauri 2                                │
│ Backend Language    │ Rust (stable)                          │
│ UI Framework        │ Vue 3 (Composition API + <script setup>)│
│ Language            │ TypeScript 5                           │
│ Styling             │ TailwindCSS 3 → 4 (upgrade planejado)  │
│ UI Components       │ Radix Vue + shadcn/vue                  │
│ State Management    │ Pinia                                   │
│ Routing             │ Vue Router 4                            │
│ Terminal            │ xterm.js                               │
│ Build Tool          │ Vite 5                                  │
│ Docker Client       │ bollard (Rust crate)                   │
│ Persistência        │ tauri-plugin-store                     │
│ Keychain            │ keyring-rs                              │
│ Testes Backend      │ cargo test + tokio-test                │
│ Testes Frontend     │ Vitest + Vue Test Utils                │
│ CI/CD               │ GitHub Actions                         │
│ Release / Build     │ tauri-action (builds multiplataforma)  │
│ Linting             │ ESLint + Clippy (Rust)                 │
│ Formatação          │ Prettier + rustfmt                     │
├─────────────────────┴───────────────────────────────────────┤
│ Repositório: GitHub público | Licença: MIT                     │
└─────────────────────────────────────────────────────────────┘
```

### 10.4 Curva de Aprendizado Estimada

| Tecnologia | Nível Inicial (Laravel Dev) | Tempo para Produtividade |
|---|---|---|
| Vue 3 Composition API | Baixo (análogo ao TALL) | 1-2 semanas |
| TypeScript (básico) | Baixo-Médio | 1-2 semanas |
| Tauri 2 (conceitos) | Médio | 1 semana |
| Rust (básico, Tauri commands) | Médio-Alto | 3-6 semanas |
| TailwindCSS | Zero (já dominado) | — |
| Pinia | Muito baixo | 1-2 dias |
| Vite | Muito baixo (já usa no Laravel) | 1-2 dias |
| bollard / Docker API | Baixo-Médio | 1-2 semanas |

**Sequência de aprendizado recomendada:**
1. Vue 3 (Crash Course oficial — ~8h)
2. TypeScript fundamentals para Vue (~4h)
3. Tauri 2 Getting Started + exemplos (~1 semana)
4. Rust básico (The Rust Book — capítulos 1-10) — pode ser paralelo ao dev
5. bollard + Docker API (documentação oficial Docker + exemplos bollard)

---

## 11. Roadmap e Faseamento

### Fase 0 — Foundation (Semanas 1-4)
**Objetivo:** Ambiente de desenvolvimento funcionando, arquitetura base, CI/CD open-source.

- [ ] Setup do projeto Tauri 2 + Vue 3 + TailwindCSS + TypeScript
- [ ] Repositório público no GitHub + licença MIT + CONTRIBUTING.md
- [ ] Configuração do GitHub Actions para build multiplataforma (macOS universal, Linux x64, Windows x64)
- [ ] Design system base (tokens, componentes primitivos com shadcn/vue)
- [ ] Integração básica com Docker Engine API (listar containers, start/stop)
- [ ] Sistema de configuração persistente (tauri-plugin-store)
- [ ] Sistema de roteamento e layout principal da UI
- [ ] System tray básico

### Fase 1 — MVP Core (Semanas 5-12)
**Objetivo:** Produto utilizável por um dev real, open-source e publicado no GitHub Releases.

**Entrega:** Gerência da Global Stack + projeto TALL Stack + Inertia.js completo funcionando.

- [ ] M1: Global Stack completo (Redis, PostgreSQL, Mailpit básicos)
- [ ] M2: Cadastro e scaffold de projetos — TALL Stack + Inertia.js + Vue 3 + Inertia.js + React
- [ ] M2: RF-M2.03.1 — Configuração de startup do Vite por projeto (automático/manual)
- [ ] M2: RF-M2.07 — Editor visual de php.ini com docs inline
- [ ] M2: RF-M2.08 — Gerenciamento visual do Supervisor (add/remove/edit workers)
- [ ] M2: RF-M2.09 — Xdebug toggle + geração de launch.json (VS Code) + config PhpStorm
- [ ] M3: Quick Actions básicas (migrate, tinker, clear caches, start Vite)
- [ ] M4: Visualizador de logs em tempo real
- [ ] M5: RF-M5.01 — Editor de docker-compose.yml com suporte a v2 legado
- [ ] M6: RF-M6.01 — Criação automática do banco de testes `{projeto}_testing`
- [ ] M7: Configurações básicas (tema, Docker socket, atalhos)
- [ ] M7: System tray com indicador de status
- [ ] Publicação da v0.1.0 no GitHub Releases

### Fase 2 — Full Feature (Semanas 13-24)
**Objetivo:** Produto completo cobrindo todos os módulos e colaboração em times.

- [ ] M1: Catálogo completo de serviços globais (MinIO, Soketi, Meilisearch, Typesense)
- [ ] M2: Wizard de novo projeto com todos os templates
- [ ] M2: RF-M5.05 — Sincronização de `.orcker.json` via Git para times
- [ ] M3: Command Palette, Favoritos, Histórico completo
- [ ] M4: Dashboard de recursos com gráficos de série temporal
- [ ] M4: Notificações de status (OS notifications)
- [ ] M5: RF-M5.06 — Marketplace de templates da comunidade (GitHub-based)
- [ ] M5: Versionamento Git de configs com diff visual e rollback
- [ ] M6: Gerenciamento de banco (dump/restore, visualizador de conexões)
- [ ] M7: Export/Import de configurações
- [ ] **M8: RF-M8.01 — Ativação de Sandbox por projeto (Cloudflare Tunnel bundled)**
- [ ] **M8: RF-M8.02 — URL pública HTTPS + QR Code gerado localmente**
- [ ] **M8: RF-M8.03 — Suporte multi-provedor (Cloudflare, ngrok, Expose)**
- [ ] **M8: RF-M8.04 — Controles de segurança (senha, expiração, allowlist IP, modo read-only)**
- [ ] **M8: RF-M8.05 — Log de acesso em tempo real com geolocalização local e export CSV**

### Fase 3 — Polish & Growth (Semanas 25-36)
**Objetivo:** Polimento, performance, extensibilidade e suporte a arquiteturas adicionais.

- [ ] Auto-updater via GitHub Releases (Tauri Updater)
- [ ] Onboarding guiado para novos usuários
- [ ] Suporte a perfis de ambiente (dev / staging local)
- [ ] Plugin system (extensões de serviço da comunidade via WASM/JS)
- [ ] Integração com Laravel Telescope (visualizador de queries e jobs)
- [ ] Internacionalização completa (PT-BR / EN)
- [ ] Linux ARM64 — build e testes em arquitetura arm64
- [ ] Suporte a múltiplos arquivos Compose por projeto (`docker-compose.override.yml`)
- [ ] **M8: RF-M8.06 — Subdomínio fixo personalizado via orcker.dev (conta gratuita + Cloudflare DNS)**

---

## 12. Riscos, Restrições e Mitigações

| # | Risco | Probabilidade | Impacto | Mitigação |
|---|---|---|---|---|
| R01 | Rust tem curva de aprendizado alta, podendo gerar atrasos | Alta | Alto | Limitar o Rust ao mínimo necessário (Tauri commands); lógica de negócio em TypeScript quando possível |
| R02 | Divergências de comportamento entre Docker Desktop, OrbStack e Colima no macOS | Média | Médio | Abstração do socket path como configuração; testes em todas as plataformas desde cedo |
| R03 | Docker Engine API muda entre versões | Baixa | Médio | Fixar versão mínima da API (v1.44); usar negociação de versão via `bollard` |
| R04 | Performance do xterm.js degradada com logs de alto volume | Média | Médio | Buffer com limite de linhas (ex.: 5000), virtualização de linhas, throttling de render |
| R05 | Escopo creep (feature bloat) sem equipe para gerenciar | Alta | Alto | MVP estrito; backlog disciplinado; "YAGNI" como princípio central de decisão |
| R06 | Conflito de portas entre projetos não detectado | Média | Médio | Validação ativa de portas antes de qualquer start; scan de portas em uso |
| R07 | Falha silenciosa no healthcheck de containers dependentes | Média | Alto | Implementar timeout + retry com feedback visual explícito; não confiar só no `depends_on` |
| R08 | Dados de configuração corrompidos após atualização | Baixa | Alto | Versionamento do schema de configuração + migration automática; backup antes de update |
| R09 | Divergência entre arquivos de infra editados fora do Orcker e estado gerenciado (sync conflict) | Média | Médio | Algoritmo de diff na abertura do projeto; avisos visuais não-bloqueantes; nunca sobrescrever silenciosamente |
| R10 | Baixa adoção open-source por falta de documentação e onboarding para contribuidores | Média | Médio | CONTRIBUTING.md detalhado desde a Fase 0; documentação de arquitetura pública; `good first issue` labels desde o MVP |
| R11 | Sandbox ativo inadvertidamente expõe projeto sensível (dados de banco, variáveis de ambiente) | Média | Alto | Badge visual permanente em toda a UI enquanto ativo; timeout obrigatório sem opção "ilimitado"; aviso de segurança proeminente ao ativar; encerramento automático ao fechar o Orcker |
| R12 | Binário `cloudflared` bundled desatualizado ou comprometido | Baixa | Alto | Verificação de hash SHA-256 do binário a cada execução; processo de atualização automatizado via CI/CD do Orcker; canal de atualização separado para o binário em caso de vulnerabilidade crítica |

---

## 13. Decisões de Produto — Refinamento §13 (Resolvido)

Esta seção registra as decisões tomadas na sessão de refinamento do PRD `0.1-draft → 0.2-refined`. Todas as questões estão resolvidas e incorporadas nos requisitos funcionais, não-funcionais e roadmap.

| # | Questão | Decisão | Impacto no PRD |
|---|---|---|---|
| Q1 | Modelo de distribuição | **Open-source MIT** — repositório público desde o MVP | §1.4, §3.1 (OBJ-05), §6 (RNF-06), §11 (Fase 0), §12 (R10) |
| Q2 | Escopo de usuários | **Times** — compartilhamento de config via `.orcker.json` no Git | §1.2, §1.4, §4 (Persona 2), RF-M5.05 |
| Q3 | Geração de arquivos de infra | **Editáveis livremente** fora do Orcker — sync com diff não-bloqueante ao abrir | §1.4, RF-M5.05, §12 (R09) |
| Q4 | Docker Compose v2 legado | **Suporte a v2 e atual** — detecção automática do binário no host | RF-M2.03, RF-M5.01 |
| Q5 | Banco de testes automático | **Sim** — `{projeto}_testing` criado automaticamente ao registrar projeto | RF-M6.01 |
| Q6 | Vite Dev Server | **Configurável por projeto** — automático ou manual; sempre no container `app` porta 5173 | RF-M2.03.1 |
| Q7 | Suporte a Inertia.js e Filament | **Desde o MVP** — templates Inertia.js + Vue 3, Inertia.js + React, Filament v3/v4 (Livewire v3/v3.5) e Filament v5 (Livewire v4) no wizard | RF-M2.03, RF-M5.04 |
| Q8 | Gerenciamento do php.ini | **Editor visual** — diretivas por categoria com documentação inline + modo raw | RF-M2.07 |
| Q9 | Supervisor | **Visual completo** — adicionar/remover/editar workers, prioridade, numprocs | RF-M2.08 |
| Q10 | Xdebug | **Toggle + VS Code (`launch.json`) + PhpStorm** — detecção automática do IDE | RF-M2.09 |
| Q11 | Marketplace de templates | **Sim** — GitHub-based, comunidade via Pull Request, exibido no wizard de novo projeto | RF-M5.06, §11 (Fase 2) |
| Q12 | Integração cloud (Forge/Vapor) | **Fora do escopo** — foco 100% local/dev; removido do roadmap | §4 (Out of Scope), §11 (Fase 3) |
| Q13 | Linux ARM64 | **MVP: x64 apenas** — ARM64 planejado para Fase 3 | §6 (RNF-02), §11 (Fase 3) |
| Q14 | Domínio | **orcker.dev** | §1 (cabeçalho) |

---

## 14. Glossário Técnico

| Termo | Definição |
|---|---|
| **TALL Stack** | TailwindCSS + AlpineJS + Livewire + Laravel |
| **HMR / Hot Reload** | Hot Module Replacement — atualização do browser em tempo real sem reload completo durante desenvolvimento frontend |
| **Supervisor** | Gerenciador de processos Unix que mantém múltiplos processos rodando dentro de um container |
| **PHP-FPM** | PHP FastCGI Process Manager — gerenciador de processos PHP para comunicação via FastCGI com o Nginx |
| **FastCGI** | Protocolo de comunicação entre servidor web (Nginx) e runtime de aplicação (PHP-FPM) |
| **Bollard** | Crate Rust para comunicação com Docker Engine API via socket |
| **Tauri IPC** | Inter-Process Communication do Tauri — canal de comunicação entre o frontend (Vue) e o backend (Rust) |
| **Healthcheck** | Verificação periódica da saúde de um serviço Docker (ex.: `pg_isready` para PostgreSQL) |
| **Pinia** | Biblioteca de gerenciamento de estado reativa para Vue 3 (substituta do Vuex) |
| **Keychain** | Armazenamento seguro de credenciais fornecido pelo sistema operacional |
| **Named Pipe** | Mecanismo de comunicação inter-processo no Windows, equivalente ao Unix Socket no Linux/macOS |
| **Scaffold** | Geração automática de estrutura de arquivos e código para um novo projeto ou componente |
| **Global Stack** | Conjunto de serviços Docker compartilhados entre todos os projetos (Redis, PostgreSQL, Mailpit, etc.) |
| **Project Stack** | Conjunto de serviços Docker específicos de um projeto (PHP-FPM, Nginx, etc.) |
| **Filament** | Framework de painel admin para Laravel baseado em Livewire e TailwindCSS. v3/v4 utiliza Livewire v3/v3.5; v5 utiliza Livewire v4 com integração Vite nativa |
| **Reverse Tunnel** | Técnica que expõe uma porta local à internet através de uma conexão de saída para um relay externo, sem necessidade de abrir portas no roteador ou ter IP público |
| **cloudflared** | Binário oficial da Cloudflare que estabelece um túnel reverso seguro (`Cloudflare Tunnel`) entre o host local e a rede da Cloudflare, provisionando HTTPS automaticamente |
| **Expose (BeyondCode)** | Ferramenta open-source Laravel-native de túnel reverso que pode ser auto-hospedada, criada por Marcel Pociot (BeyondCode) |
| **Sandbox** | Sessão temporária e protegida do Orcker que expõe um projeto local à internet via túnel reverso para fins de demonstração, homologação ou validação com clientes e empregadores |
| **GeoIP local** | Banco de dados de geolocalização de IPs armazenado localmente no Orcker (ex.: MaxMind GeoLite2), utilizado para identificar o país de origem de acessos sem chamadas externas de API |

---

## 15. Referências

- Wallace Martins — *Docker para Desenvolvimento Laravel: O Setup Que Eu Uso em Produção* (Nov 2025) — Documento de referência técnica base para este PRD.
- Documentação oficial Tauri 2: https://tauri.app
- Documentação oficial Vue 3: https://vuejs.org
- Docker Engine API Reference v1.44: https://docs.docker.com/engine/api/v1.44/
- bollard Rust crate: https://docs.rs/bollard/latest/bollard/
- Radix Vue: https://www.radix-vue.com
- shadcn/vue: https://www.shadcn-vue.com
- xterm.js: https://xtermjs.org
- Cloudflare Tunnel (cloudflared): https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/
- ngrok: https://ngrok.com/docs
- Expose (BeyondCode): https://expose.dev
- MaxMind GeoLite2 (GeoIP local): https://dev.maxmind.com/geoip/geolite2-free-geolocation-data

---

*PRD versão 0.3.1 — Filament v3/v4/v5 incorporado como stack de scaffolding. 8 módulos funcionais · 39+ requisitos funcionais · 12 riscos mapeados.*
*Próximo passo: Software Architecture Document (SAD) — contratos de IPC, estrutura de módulos Rust, schema do `.orcker.json` e integração do `cloudflared`.*
