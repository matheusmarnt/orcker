# Changelog

## [0.3.0](https://github.com/matheusmarnt/orcker/compare/orcker-v0.2.0...orcker-v0.3.0) (2026-05-15)


### Features

* **config:** hardens CSP, capabilities mínimas e pubkey Ed25519 do updater ([c31bbee](https://github.com/matheusmarnt/orcker/commit/c31bbeea472a5eabfc871cda037a4ae99e1abfe2))
* **m1-global:** adiciona adapter de rede Docker (ensure/remove orcker-global) ([3dd053c](https://github.com/matheusmarnt/orcker/commit/3dd053ce6bad285314a021c65e356f38bb5e6fd3))
* **m1-global:** adiciona plugin-global-shortcut, plugin-store e shadcn Switch ([5485496](https://github.com/matheusmarnt/orcker/commit/548549606565e52fcd71bc1a1914f1091e134806))
* **m1-global:** adiciona useGlobalStackStore com testes Vitest ([f173103](https://github.com/matheusmarnt/orcker/commit/f173103ae28b58a6337445ceef0c31815adb9df3))
* **m1-global:** define ServiceId, ServiceConfig, ServiceStatus e GlobalStackState ([6c528a4](https://github.com/matheusmarnt/orcker/commit/6c528a40296f26fc2bb4a9463857273165359ed5))
* **m1-global:** implementa 5 comandos Tauri do Global Stack e estende AppState ([2133ef9](https://github.com/matheusmarnt/orcker/commit/2133ef9e9e7eebcc957216013993009c1e9b1114))
* **m1-global:** implementa UI Global Stack — ServiceCard, ConfigPanel, view e rota ([5c357bd](https://github.com/matheusmarnt/orcker/commit/5c357bd8c760e446f3c27e3e0520ed70ca36f629))
* **m1-global:** registra tauri-plugin-store e tauri-plugin-global-shortcut no builder ([f87f8ca](https://github.com/matheusmarnt/orcker/commit/f87f8ca78f65b7022dfe53f4ac3b817727fccb9d))
* **rust:** scaffolda módulos Rust da Phase 1 com bollard, AppState e AppError ([413a01b](https://github.com/matheusmarnt/orcker/commit/413a01b4be4456837e4d5ffecab1aedc2ab51c2d))
* **rust:** wire tauri-specta Builder, tracing e probe_docker em lib.rs ([16abf45](https://github.com/matheusmarnt/orcker/commit/16abf4548f355d5276a947760f800fbd7f8fe4db))
* **ui:** adiciona ContainerTable, DockerStatusBadge e ErrorScreen ([edb897f](https://github.com/matheusmarnt/orcker/commit/edb897f58846895399b1aef15091f0bb59deab1f))
* **ui:** bootstrap Vue app com Pinia, router e camada IPC tipada ([b28cf2b](https://github.com/matheusmarnt/orcker/commit/b28cf2b1d6aef44b812f9e63ac0e1107d521528d))
* **ui:** instala deps frontend e inicializa shadcn-vue com reka-ui ([615b51c](https://github.com/matheusmarnt/orcker/commit/615b51c44b1c6a02a562d55bf513d53beb598bd2))
* **ui:** wire DashboardView com skeleton, tabela e ErrorScreen via Pinia store ([0d4aaa0](https://github.com/matheusmarnt/orcker/commit/0d4aaa0b0cdc215ad3df984bf58b147389c06241))


### Bug Fixes

* **m1-global:** auto-pull imagem Docker antes de criar container e adiciona log de erro ([ffb68de](https://github.com/matheusmarnt/orcker/commit/ffb68de1d8260785db0758aa0cbbe5a47f84ccf9))
* **m1-global:** corrige API do Switch para reka-ui v2 e alinhamento flex dos cards ([bc846d9](https://github.com/matheusmarnt/orcker/commit/bc846d928f6d0bf0784b00323b828c3e1f381148))
* **m1-global:** corrige inicialização do PostgreSQL no Docker ([898d8d0](https://github.com/matheusmarnt/orcker/commit/898d8d0e0a39cc05169683270fa69e7662022c6b))
* **m1-global:** corrige sistema de toast — CSS, contexto Vue e transições ([9ab830a](https://github.com/matheusmarnt/orcker/commit/9ab830ab15678965510149f5bf3261e4fafaa579))
* **rust:** adiciona delay 500ms antes do primeiro probe para sincronizar listeners Vue ([78f4aa0](https://github.com/matheusmarnt/orcker/commit/78f4aa08c5387d56a3150c00953de53d7bfcf4e7))
* **rust:** colapsa if let aninhado em stop_service (clippy::collapsible_match) ([0737a08](https://github.com/matheusmarnt/orcker/commit/0737a080af70539926cf7700bf5875a99203bfd0))
* **rust:** corrige tipo fn pointer em teste de compilação de networks (async fn não coerce para fn pointer) ([bc07956](https://github.com/matheusmarnt/orcker/commit/bc0795682acbaad4e9c43d3e9b4ad833911c2344))
* **rust:** remove tauri_plugin_log — conflita com tracing_log::LogTracer no log global ([f7e367a](https://github.com/matheusmarnt/orcker/commit/f7e367aea4c3c753a507099c54a17441ec4a103e))
* **rust:** usa try_init em tracing_subscriber para evitar conflito com tauri_plugin_log ([edc19a1](https://github.com/matheusmarnt/orcker/commit/edc19a1170462694053e01a429d32258d8eb6db4))
* **ui:** adiciona Toaster global e corrige config panel com v-if ([1b6ed1d](https://github.com/matheusmarnt/orcker/commit/1b6ed1dcad7d6beb3b8472792aa17ef85b0aad55))
* **ui:** move listener init para App setup e adiciona fallback de estado ([ef683ad](https://github.com/matheusmarnt/orcker/commit/ef683ad85250b6bca1fc0eed4afb9c13cde8258d))

## [0.2.0](https://github.com/matheusmarnt/orcker/compare/orcker-v0.1.0...orcker-v0.2.0) (2026-05-10)


### Features

* **ui:** scaffold Tauri 2 + Vue 3 + TypeScript base app ([bf9e5cc](https://github.com/matheusmarnt/orcker/commit/bf9e5ccf4307150a604c92af2612e8858447243a))


### Bug Fixes

* **ci:** correct release-please version strategy to stay pre-1.0.0 ([114e4dc](https://github.com/matheusmarnt/orcker/commit/114e4dc6f3a98ba57250639269721524d4a45f16))
* **ci:** pass vitest when no test files exist yet ([4259675](https://github.com/matheusmarnt/orcker/commit/4259675049c31289ea347386071f8e5800ee0acb))
* **ci:** resolve eslint errors in vite-env.d.ts and approve esbuild build scripts ([4255737](https://github.com/matheusmarnt/orcker/commit/4255737e857387968cf1cc14ffba7819c9ddea94))
* **ci:** set initial-version 0.1.0 to prevent release-please proposing 1.0.0 ([28cc5e4](https://github.com/matheusmarnt/orcker/commit/28cc5e454e3357adb43462ecdb63667b42e108c9))

## 0.1.0 (2026-05-10)


### Features

* **ui:** scaffold Tauri 2 + Vue 3 + TypeScript base app ([0d7e1fc](https://github.com/matheusmarnt/orcker/commit/0d7e1fc95ff175868cf0e29a9f347461c253296d))


### Bug Fixes

* **ci:** correct release-please version strategy to stay pre-1.0.0 ([1b108b5](https://github.com/matheusmarnt/orcker/commit/1b108b586439b4249d9977ea882ad615c9b92ee9))
* **ci:** pass vitest when no test files exist yet ([5965e49](https://github.com/matheusmarnt/orcker/commit/5965e4991f0badba68ab34b56e01c90d8dec8409))
* **ci:** resolve eslint errors in vite-env.d.ts and approve esbuild build scripts ([82cbaa4](https://github.com/matheusmarnt/orcker/commit/82cbaa4fdc11b4e3d76f05406fa163789f33a46e))
* **ci:** set initial-version 0.1.0 to prevent release-please proposing 1.0.0 ([ba11e1d](https://github.com/matheusmarnt/orcker/commit/ba11e1dd55f0656ebbd07fa423e4101f87315548))

## Changelog

All notable changes to Orcker will be documented in this file.

This file is auto-generated by [Release Please](https://github.com/googleapis/release-please). Do not edit manually.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/). Versioning follows [Semantic Versioning](https://semver.org/).
