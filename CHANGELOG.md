# Changelog

## [0.3.0](https://github.com/matheusmarnt/orcker/compare/orcker-v0.2.0...orcker-v0.3.0) (2026-05-15)


### Features

* **config:** harden CSP, minimal capabilities and Ed25519 pubkey for updater ([29e7fa4](https://github.com/matheusmarnt/orcker/commit/29e7fa450dc913ebdf403c480ce55d35564cff42))
* **m1-global:** add Docker network adapter (ensure/remove orcker-global) ([edb50b4](https://github.com/matheusmarnt/orcker/commit/edb50b484b2a228c7965838784a2f0db948900fb))
* **m1-global:** add plugin-global-shortcut, plugin-store and shadcn Switch ([0cd5ec6](https://github.com/matheusmarnt/orcker/commit/0cd5ec6cc624fd50e0f20aadd21bdd029498274a))
* **m1-global:** add useGlobalStackStore with Vitest tests ([4d5c726](https://github.com/matheusmarnt/orcker/commit/4d5c726d281df4e27d0391721b7d863138337c12))
* **m1-global:** define ServiceId, ServiceConfig, ServiceStatus and GlobalStackState ([a74cb54](https://github.com/matheusmarnt/orcker/commit/a74cb543bf09477019e87fc3de0318132590715c))
* **m1-global:** implement 5 Tauri commands for Global Stack and extend AppState ([c17536c](https://github.com/matheusmarnt/orcker/commit/c17536c4034d4d5dab6b772dc1a94a95cf4d096e))
* **m1-global:** implement Global Stack UI — ServiceCard, ConfigPanel, view and route ([bea4f68](https://github.com/matheusmarnt/orcker/commit/bea4f68875104c0f589aa818a362f6f871f1e97b))
* **m1-global:** register tauri-plugin-store and tauri-plugin-global-shortcut in builder ([e7721e2](https://github.com/matheusmarnt/orcker/commit/e7721e2fb7269745a9da0e817d3f9e4b5fbafe6b))
* **rust:** scaffold Phase 1 Rust modules with bollard, AppState and AppError ([bed6a6c](https://github.com/matheusmarnt/orcker/commit/bed6a6ccde0e7c2d82c2b163342134a5adfea93f))
* **rust:** wire tauri-specta Builder, tracing and probe_docker in lib.rs ([12b86e4](https://github.com/matheusmarnt/orcker/commit/12b86e422e07b89ea2ef55fc63022242609b8d41))
* **ui:** add ContainerTable, DockerStatusBadge and ErrorScreen ([22df46f](https://github.com/matheusmarnt/orcker/commit/22df46faf9d25d38020fc4d974884ef100712b02))
* **ui:** bootstrap Vue app with Pinia, router and typed IPC layer ([f0d1d27](https://github.com/matheusmarnt/orcker/commit/f0d1d2708bdd205eb0b6283496613a8058cb04a9))
* **ui:** install frontend deps and initialize shadcn-vue with reka-ui ([11818ae](https://github.com/matheusmarnt/orcker/commit/11818ae2e8a1f049936f3a373e179a88d43472a6))
* **ui:** wire DashboardView with skeleton, table and ErrorScreen via Pinia store ([1b3763c](https://github.com/matheusmarnt/orcker/commit/1b3763cf0bc6d0b9cdbb1315af95da698bef55d4))


### Bug Fixes

* **m1-global:** auto-pull Docker image before creating container and add error logging ([f0068b8](https://github.com/matheusmarnt/orcker/commit/f0068b89121607d0c21ac5f8e38097ff2f11da76))
* **m1-global:** fix PostgreSQL container startup with env vars and fail-fast polling ([fa30880](https://github.com/matheusmarnt/orcker/commit/fa30880db85c5fde22fb3fb35cfbbb045086358d))
* **m1-global:** fix Switch API for reka-ui v2 and flex alignment of cards ([9422dfc](https://github.com/matheusmarnt/orcker/commit/9422dfc6d2eae93cacaa2c3cec16487ebabaf83a))
* **m1-global:** fix toast system — CSS import, Vue context and state transitions ([0df931f](https://github.com/matheusmarnt/orcker/commit/0df931fc92901009767ea2fade57a2a3dd33d6df))
* **rust:** add 500ms delay before first probe to sync Vue listeners ([9ac0c7c](https://github.com/matheusmarnt/orcker/commit/9ac0c7c463be6ed3283ae1319f9a68072ddaa408))
* **rust:** collapse nested if let in stop_service (clippy::collapsible_match) ([488c4a8](https://github.com/matheusmarnt/orcker/commit/488c4a8f172b2d3cf0078de9e210b003eeb32e51))
* **rust:** remove fn pointer coercion for async fn in network compilation test ([6a19130](https://github.com/matheusmarnt/orcker/commit/6a19130d77edce99013183a6601f36a532a03007))
* **rust:** remove tauri_plugin_log — conflicts with tracing_log::LogTracer ([33603ad](https://github.com/matheusmarnt/orcker/commit/33603ad4646e1b4edb1a7bd66e741c0b989ab9eb))
* **rust:** use try_init in tracing_subscriber to avoid conflict with tauri_plugin_log ([c1479be](https://github.com/matheusmarnt/orcker/commit/c1479be8d8cd1cb5d92998832433daf802f0dbe3))
* **ui:** add global Toaster and fix config panel with v-if ([9f46b6d](https://github.com/matheusmarnt/orcker/commit/9f46b6dd9c6170ededa6befdc91c9ebb7bdd6567))
* **ui:** move listener init to App setup and add state fallback ([f135c47](https://github.com/matheusmarnt/orcker/commit/f135c472086b8aedb04f54828c3cb7ae90037aa3))

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
