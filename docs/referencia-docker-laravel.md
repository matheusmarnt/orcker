# Referência Técnica: Ambiente Docker para Aplicações Laravel

> **Fonte:** [Docker para Desenvolvimento Laravel: O Setup Que Eu Uso em Produção](https://wallacemartinss.dev/pt/blog/docker-for-laravel-development)
> **Autor:** Wallace Martins
> **Data:** 21 Nov 2025
> **Leitura estimada:** 18 min

---

## Sumário

1. [Contexto e Motivação](#contexto-e-motivacao)
2. [Arquitetura Geral](#arquitetura-geral)
3. [Serviços e Responsabilidades](#servicos-e-responsabilidades)
4. [Orquestração com Docker Compose](#orquestracao-com-docker-compose)
5. [Estratégia de Redes](#estrategia-de-redes)
6. [Build da Aplicação (Dockerfile.dev)](#build-da-aplicacao)
7. [Gerenciamento de Processos com Supervisor](#gerenciamento-de-processos-com-supervisor)
8. [Configuração do Nginx](#configuracao-do-nginx)
9. [Configuração do PHP](#configuracao-do-php)
10. [Inicialização do PostgreSQL](#inicializacao-do-postgresql)
11. [Estrutura de Diretórios](#estrutura-de-diretorios)
12. [Fluxo de Inicialização](#fluxo-de-inicializacao)
13. [Comandos Operacionais](#comandos-operacionais)
14. [Boas Práticas e Decisões Técnicas](#boas-praticas-e-decisoes-tecnicas)
15. [Referências e Tecnologias](#referencias-e-tecnologias)

---

## Contexto e Motivação

### Problema que o setup resolve

O ambiente Docker para Laravel foi concebido para eliminar três problemas recorrentes em times de desenvolvimento:

- **Ambientes inconsistentes** entre desenvolvedores ("funciona na minha máquina")
- **Conflitos de versão** de PHP, extensões e banco de dados
- **Divergência entre desenvolvimento e produção**, causando surpresas no deploy

### Por que não usar Laravel Sail?

O Laravel Sail funciona bem para projetos simples, mas não oferece controle granular sobre:

- Imagem base do container
- Configuração do Nginx
- Gerenciamento de processos com Supervisor
- Customização de extensões PHP

Para esses cenários, um setup próprio é necessário.

---

## Arquitetura Geral

O ambiente é composto por **três containers** que se comunicam via rede Docker interna:

| Container | Tecnologia | Responsabilidade |
|-----------|-----------|-----------------|
| **app** | PHP-FPM + Supervisor | Executa o Laravel e seus workers |
| **nginx** | Nginx Alpine | Reverse proxy para requisições HTTP |
| **postgres** | PostgreSQL | Banco de dados com persistência |

### Fluxo de Comunicação

Browser → Nginx (porta 80) → PHP-FPM via FastCGI (porta 9000 interna) → Laravel

> O browser nunca acessa o PHP diretamente. A comunicação Nginx e PHP-FPM ocorre via protocolo FastCGI na rede interna Docker.

---

## Serviços e Responsabilidades

### App (PHP-FPM)

- Imagem base: php:8.4-rc-fpm
- Gerenciado pelo Supervisor (múltiplos processos internos)
- Expõe a porta **5173** para o Vite (hot-reload de frontend)
- Conectado a duas redes: interna (app-network) e compartilhada (development)

### Nginx

- Imagem: nginx:alpine
- Recebe requisições na porta **80**
- Passa para o PHP-FPM via FastCGI (app:9000)
- Serve arquivos estáticos diretamente (assets, storage público)
- Configuração via template com variável de ambiente FASTCGI_PASS

### PostgreSQL

- Imagem: postgres:latest
- Expõe a porta **5432**
- Variáveis de ambiente compartilhadas via .env do Laravel
- Volume persistente para dados (pgsql-data)
- Healthcheck para garantir disponibilidade antes de aceitar conexões

---

## Orquestração com Docker Compose

### Decisões de design

| Decisão | Justificativa |
|---------|--------------|
| restart: no em desenvolvimento | Falhas visíveis ao invés de reinício silencioso |
| env_file: .env no PostgreSQL | Uma única fonte de verdade para credenciais |
| healthcheck no banco | Evita falhas silenciosas durante migrações |
| Porta 5173 exposta no app | Necessária para hot-reload do Vite |
| FASTCGI_PASS como variável de ambiente | Permite reutilizar o mesmo default.conf em múltiplos ambientes |

### Volumes mapeados

| Serviço | Volume Host | Volume Container | Modo |
|---------|------------|-----------------|------|
| app | ./ | /var/www/ | rw |
| nginx | ./public | /var/www/public | rw |
| nginx | ./storage/app/public | /var/www/storage/app/public | ro |
| postgres | pgsql-data (named) | /var/lib/postgresql/data | rw |

---

## Estratégia de Redes

O setup utiliza **duas redes Docker** por projeto:

### app-network (interna)
- Isolada por projeto
- Conecta: app, nginx e postgres
- Tipo: bridge

### development (compartilhada/externa)
- Compartilhada entre todos os projetos da máquina
- Permite acesso a serviços globais: Redis, Mailpit, MinIO, Soketi
- Tipo: bridge external

### Configuração no .env do Laravel para serviços compartilhados

REDIS_HOST=redis
MAIL_HOST=mailpit
AWS_ENDPOINT=http://minio:9000

### Benefícios da abordagem de rede compartilhada

- **Economia de recursos**: um Redis serve N projetos
- **Consistência**: todos os projetos usam a mesma versão dos serviços
- **Simplicidade**: docker-compose.yml de cada projeto permanece enxuto
- **Velocidade**: serviços compartilhados já estão rodando ao iniciar novo projeto

Criação da rede externa (executar uma vez):
  docker network create development

---

## Build da Aplicação (Dockerfile.dev)

### Visão geral das etapas

O Dockerfile segue uma estrutura modular com as seguintes etapas:

#### 1. Imagem base e configuração inicial

- Base: php:8.4-rc-fpm (RC para antecipar breaking changes antes da produção)
- Criação de usuário com UID igual ao do host (evita conflitos de permissão)
- DEBIAN_FRONTEND=noninteractive para builds não-interativos

#### 2. Repositório PostgreSQL

- Adiciona o repositório oficial para garantir o client PostgreSQL v18
- Necessário para pg_dump, psql e compatibilidade com o servidor

#### 3. Dependências do sistema

| Dependência | Finalidade |
|------------|------------|
| libpq-dev | Driver PostgreSQL |
| libicu-dev | Internacionalização |
| libpng-dev, libjpeg62-turbo-dev, libwebp-dev | Manipulação de imagens |
| libfreetype6-dev | Renderização de fontes |
| default-mysql-client | Conexão com MySQL quando necessário |
| postgresql-client-18 | Operações de banco via CLI |
| openssh-client | Operações Git via SSH no container |
| supervisor | Gerenciamento de múltiplos processos |

Node.js 20 LTS instalado para compilação de assets (Vite).
Flags --no-install-recommends e remoção de listas de pacotes mantêm a imagem enxuta.

#### 4. Extensões PHP compiladas

| Extensão | Finalidade |
|----------|-----------|
| pdo_pgsql, pgsql | Banco PostgreSQL |
| pdo_mysql | Banco MySQL |
| gd | Manipulação de imagens |
| intl | Formatação de moeda/data |
| pcntl | Gerenciamento de workers (Horizon) |
| bcmath | Cálculos financeiros precisos |
| mbstring | Manipulação de strings multibyte |
| zip, soap | Compressão e integração com SOAP |
| sockets | Comunicação via sockets |
| exif | Metadados de imagens |
| gmp | Operações matemáticas de precisão |

Compilação em paralelo com -j"$(nproc)" reduz o tempo de build significativamente.

#### 5. Composer e Redis

- Composer: instalado via multi-stage build (COPY --from=composer:latest)
- Redis: instalado via PECL

#### 6. Configuração de segurança

- Container roda como usuário não-root
- Permissões ajustadas em storage/ e bootstrap/cache/ para escrita do Laravel
- Supervisor com diretórios de log/PID atribuídos ao usuário da aplicação

#### 7. Portas expostas

| Porta | Finalidade |
|-------|-----------|
| 80 | HTTP (via Nginx) |
| 6024 | WebSocket (Reverb/Soketi) |

---

## Gerenciamento de Processos com Supervisor

O Supervisor gerencia **quatro processos** dentro do container da aplicação:

| Processo | Comando | Prioridade | Finalidade |
|---------|---------|-----------|-----------|
| php-fpm | php-fpm --nodaemonize | 5 (primeiro) | Servir a aplicação Laravel |
| laravel-horizon | php artisan horizon | 15 | Processar filas |
| laravel-schedule | Loop com schedule:run a cada 60s | — | Tarefas agendadas |
| laravel-pulse | php artisan pulse:check | 20 | Monitoramento |

### Configurações relevantes do Supervisor

- nodaemon=true: mantém o Supervisor em foreground (requisito Docker)
- startsecs=10 no Horizon: aguarda conexão com Redis antes de considerar iniciado
- stopwaitsecs=30 no Horizon: permite que jobs em execução terminem (shutdown gracioso)
- Schedule como loop infinito com sleep 60: simula o cron do Laravel

---

## Configuração do Nginx

### nginx.conf — Configuração global

| Diretiva | Valor | Justificativa |
|---------|-------|--------------|
| worker_processes | auto | Detecta CPUs automaticamente |
| worker_connections | 4096 | Alta concorrência |
| use epoll | — | I/O mais eficiente no Linux |
| server_tokens | off | Oculta versão do Nginx (segurança) |
| client_max_body_size | 50M | Suporte a uploads maiores |
| gzip_comp_level | 5 | Equilíbrio CPU × compressão |
| keepalive_timeout | 30s | Reduz overhead de conexões |

Headers de segurança aplicados: X-Frame-Options: SAMEORIGIN e X-Content-Type-Options: nosniff.

### default.conf — Virtual host Laravel

- Roteamento via try_files para arquivos estáticos e fallback para index.php
- Passagem para PHP-FPM via fastcgi_pass com variável de ambiente substituída pelo template
- Serving direto de /storage/ via alias para o diretório de storage público

---

## Configuração do PHP

### php.ini customizado

| Diretiva | Valor | Justificativa |
|---------|-------|--------------|
| memory_limit | 512M | Evita erros em seeders/jobs pesados |
| upload_max_filesize | 100M | Uploads de arquivos maiores |
| post_max_size | 100M | Alinhado com upload_max_filesize |
| max_execution_time | 300s | Jobs e processos longos |
| date.timezone | America/Sao_Paulo | Timezone do Brasil |
| opcache.enable | 1 | Caching de bytecode PHP |
| opcache.memory_consumption | 256MB | Espaço para cache |
| opcache.validate_timestamps | 1 | Detecta alterações em arquivos |
| opcache.revalidate_freq | 2s | Verifica mudanças a cada 2 segundos |

OPcache com validate_timestamps=1 oferece performance sem sacrificar hot-reload em desenvolvimento.

---

## Inicialização do PostgreSQL

O script init.sql é executado automaticamente na **primeira inicialização** do container via /docker-entrypoint-initdb.d/:

- Timezone configurado para America/Sao_Paulo
- Formato de data brasileiro (DMY)
- Text search configurado para português (pg_catalog.portuguese)
- **Banco de testes criado automaticamente**

O banco de testes pronto desde o início permite rodar php artisan test sem configuração adicional.

---

## Estrutura de Diretórios

projeto/
├── docker/
│   ├── nginx/
│   │   ├── nginx.conf
│   │   └── default.conf
│   ├── php/
│   │   └── php.ini
│   ├── postgresql/
│   │   └── init.sql
│   ├── supervisord.conf
│   └── entrypoint.sh
├── docker-compose.yml
├── Dockerfile.dev
└── .env

Todos os arquivos são versionados no Git. Qualquer desenvolvedor clona o repositório, executa docker compose up -d e tem o ambiente completo em minutos.

---

## Fluxo de Inicialização

1. Build das imagens: docker compose build
2. Subir os containers: docker compose up -d
3. Instalar dependências do PHP: docker compose exec app composer install
4. Instalar dependências do Node: docker compose exec app npm install
5. Copiar e gerar .env: docker compose exec app cp .env.example .env && php artisan key:generate
6. Executar migrações: docker compose exec app php artisan migrate --seed
7. Build do frontend: docker compose exec app npm run build
8. Acessar: http://localhost

### Hot-reload de frontend (desenvolvimento)

docker compose exec app npm run dev

O Vite sobe na porta 5173 e atualiza o browser automaticamente a cada mudança no frontend.

---

## Comandos Operacionais

### Artisan
- docker compose exec app php artisan migrate
- docker compose exec app php artisan tinker
- docker compose exec app php artisan queue:work

### Testes
- docker compose exec app php artisan test
- docker compose exec app ./vendor/bin/pest

### Code Quality
- docker compose exec app ./vendor/bin/pint

### Logs
- docker compose exec app tail -f storage/logs/laravel.log
- docker compose logs -f app
- docker compose logs -f postgres

---

## Boas Práticas e Decisões Técnicas

| Prática | Descrição |
|--------|-----------|
| restart: no em dev | Falhas ficam visíveis; reinício automático oculta problemas |
| Volumes com :ro | Nginx não precisa escrever no storage — mais seguro |
| Healthcheck no banco | depends_on sozinho garante que o container subiu, não que o serviço está pronto |
| Dockerfile separado (dev vs prod) | Dev tem Node.js, Composer, Xdebug. Produção tem o mínimo necessário |
| Redes nomeadas | Permitem coexistência de múltiplos projetos Docker na mesma máquina |
| Usuário não-root | Segurança: o container nunca roda com privilégios de root |
| UID igual ao host | Evita conflitos de permissão entre arquivos criados no container e no host |
| set -eux no Dockerfile | Build falha ruidosamente se qualquer extensão não compilar |

---

## Referências e Tecnologias

### Stack técnica abordada

| Tecnologia | Versão | Papel |
|-----------|--------|-------|
| PHP | 8.4 RC | Runtime da aplicação |
| Laravel | — | Framework PHP |
| PHP-FPM | — | Gerenciador de processos PHP |
| Nginx | Alpine | Reverse proxy / web server |
| PostgreSQL | Latest (client v18) | Banco de dados |
| Supervisor | — | Gerenciador de processos |
| Composer | Latest | Gerenciador de dependências PHP |
| Node.js | 20 LTS | Runtime para build de assets |
| Vite | — | Bundler e hot-reload |
| Redis | — | Cache e filas (serviço compartilhado) |
| Mailpit | — | Captura de e-mails em dev (serviço compartilhado) |
| MinIO | — | Storage S3-compatível (serviço compartilhado) |
| Laravel Horizon | — | Dashboard e gestão de filas |
| Laravel Pulse | — | Monitoramento de performance |
| Laravel Reverb/Soketi | — | WebSockets |

### Tags relacionadas

Docker · Laravel · DevOps · PHP · Nginx · PostgreSQL · Supervisor · PHP-FPM

---

*Documento gerado a partir da análise do artigo original para uso como referência em planejamento de PRD.*
