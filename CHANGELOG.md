# Changelog

## [0.5.0](https://github.com/matheusmarnt/orcker/compare/orcker-v0.4.0...orcker-v0.5.0) (2026-05-16)


### Features

* **m1-global:** add MinIO, Soketi, Meilisearch to global stack ([57bdb13](https://github.com/matheusmarnt/orcker/commit/57bdb1350077b1471d1a887423716e052e2b551b))
* **m1-global:** add MySQL to global stack ([8cca5b0](https://github.com/matheusmarnt/orcker/commit/8cca5b0ebced1cdd0b9e36ce955c60b5cde72e13))
* **m2-projects:** add Filament, API-only, Jetstream scaffold templates ([5e76f90](https://github.com/matheusmarnt/orcker/commit/5e76f90f7d4928a2621aefc4183dbce81ef983de))
* **m3-terminal:** add command palette + ConfigHistory diff viewer ([bd94449](https://github.com/matheusmarnt/orcker/commit/bd944496a50189903827b1717730e32b2a47959b))
* **m4-logs:** add resource graphs and OS notifications on unhealthy project ([01fdd50](https://github.com/matheusmarnt/orcker/commit/01fdd50a5d3f337f9b61c06205cd3a9ef6501b5f))
* **m5-infra:** add git2 versioning module + get_compose_diff command ([6540ec5](https://github.com/matheusmarnt/orcker/commit/6540ec5a06d5ae0a02e8f2aa1b366eaf25e22e9e))
* **m5-infra:** add Infra view with VolumeList and ImageList components ([aeb3a57](https://github.com/matheusmarnt/orcker/commit/aeb3a57343902942fa7b3a2d5280f3ecf625249a))
* **m5-infra:** add read_compose_file + save_compose_file Tauri commands ([05ed657](https://github.com/matheusmarnt/orcker/commit/05ed6573500f45dd893aec6e8dc535e726115cfc))
* **m5-infra:** add template marketplace Rust backend + IPC bindings ([907f872](https://github.com/matheusmarnt/orcker/commit/907f8727c37461d6dc519809f8cca748f3f8b5e3))
* **m5-infra:** add TemplateMarketplace UI + CI bundle targets per platform ([6ede1be](https://github.com/matheusmarnt/orcker/commit/6ede1be31bd16b384970ad8a8a3e3689940b850c))
* **m5-infra:** add volume and image bollard adapters with Tauri commands ([cc64a4e](https://github.com/matheusmarnt/orcker/commit/cc64a4e2607729cdf10c521aaaf7b278de281c8c))
* **m5-infra:** tab navigation, pagination, resource monitor ([7e1be87](https://github.com/matheusmarnt/orcker/commit/7e1be87725b1debec6f118c9aaba940e66cd334f))
* **m6-db:** add database command stub module (R-M6.1) ([d0a14ba](https://github.com/matheusmarnt/orcker/commit/d0a14ba0c6bb0003ed5ad42f3d3da80e494aa543))
* **m6-db:** add DatabaseTab component and Database button to ProjectCard ([1c3abee](https://github.com/matheusmarnt/orcker/commit/1c3abee0dcf1acb818cfa7df5799400d3157669e))
* **m6-db:** implement database Rust commands + auto-create testing DB hook ([eb80423](https://github.com/matheusmarnt/orcker/commit/eb80423dbb20d7adf5542492a865bcd67bee05ec))
* **m7-settings:** add AppSettings struct + get/save_settings commands ([67ace2c](https://github.com/matheusmarnt/orcker/commit/67ace2c00a203613ef64192585590070ea028b40))
* **m7-settings:** add checkForUpdate composable with Sonner toast and startup wiring ([ecf1179](https://github.com/matheusmarnt/orcker/commit/ecf117915fa742436bd707ca4a5289195c11d34b))
* **m7-settings:** add Settings modal with 4 sections and gear icon entry ([7218b3f](https://github.com/matheusmarnt/orcker/commit/7218b3f9c487f69edb4fa69168a17f607de036e0))
* **m7-settings:** add useSettingsStore, vue-i18n bootstrap, and i18n messages ([91ad6d4](https://github.com/matheusmarnt/orcker/commit/91ad6d40ef46b579d33b673e1a8c6a5e6cf8178f))
* **m7-settings:** apply i18n across all components and views ([f02105c](https://github.com/matheusmarnt/orcker/commit/f02105cd15e7e94790b94dda1d5fef37432c4ff7))
* **m7-settings:** preferences tab, fix theme/locale toggle, add Spanish ([fd8048b](https://github.com/matheusmarnt/orcker/commit/fd8048bd86f5391924676ac63dc4458919ca7a1c))
* **m7-settings:** wire system tray, close-to-tray, updater and autostart plugins ([509b312](https://github.com/matheusmarnt/orcker/commit/509b312977aef187c85fb030c03adeff7fd41a4a))


### Bug Fixes

* **m5-infra:** guard null prune data in ImageList and VolumeList ([4d4fb7f](https://github.com/matheusmarnt/orcker/commit/4d4fb7f0fe58511ff9d44eafe327342958a86bad))
* **m5-infra:** prepend diff line origin char in diff_last_two ([693f925](https://github.com/matheusmarnt/orcker/commit/693f9250346dda20c1c5a65558a4597fe2bf9fb7))
* **m5-infra:** replace i64/u64 with f64 in specta-exported types ([4de2441](https://github.com/matheusmarnt/orcker/commit/4de2441e048162ac457fb00a7f3626b2dbb040fd))
* **m6-db:** wire DatabaseTab component into ProjectCard panel ([9232ca7](https://github.com/matheusmarnt/orcker/commit/9232ca7ebcb6892619f8e44239ab13e7203b1c4c))
* **m7-settings:** resolve ESLint errors from i18n pass ([f803082](https://github.com/matheusmarnt/orcker/commit/f8030821110abd4cca6fca240839033a39075904))

## [0.4.0](https://github.com/matheusmarnt/orcker/compare/orcker-v0.3.0...orcker-v0.4.0) (2026-05-15)


### Features

* **config:** harden CSP, minimal capabilities and Ed25519 pubkey for updater ([29e7fa4](https://github.com/matheusmarnt/orcker/commit/29e7fa450dc913ebdf403c480ce55d35564cff42))
* **m1-global:** add config persistence and Unhealthy status ([8d948e9](https://github.com/matheusmarnt/orcker/commit/8d948e9e6ccbd8cf7a867268063d3d89d8bc86c2))
* **m1-global:** add Docker network adapter (ensure/remove orcker-global) ([edb50b4](https://github.com/matheusmarnt/orcker/commit/edb50b484b2a228c7965838784a2f0db948900fb))
* **m1-global:** add plugin-global-shortcut, plugin-store and shadcn Switch ([0cd5ec6](https://github.com/matheusmarnt/orcker/commit/0cd5ec6cc624fd50e0f20aadd21bdd029498274a))
* **m1-global:** add useGlobalStackStore with Vitest tests ([4d5c726](https://github.com/matheusmarnt/orcker/commit/4d5c726d281df4e27d0391721b7d863138337c12))
* **m1-global:** define ServiceId, ServiceConfig, ServiceStatus and GlobalStackState ([a74cb54](https://github.com/matheusmarnt/orcker/commit/a74cb543bf09477019e87fc3de0318132590715c))
* **m1-global:** implement 5 Tauri commands for Global Stack and extend AppState ([c17536c](https://github.com/matheusmarnt/orcker/commit/c17536c4034d4d5dab6b772dc1a94a95cf4d096e))
* **m1-global:** implement Global Stack UI — ServiceCard, ConfigPanel, view and route ([bea4f68](https://github.com/matheusmarnt/orcker/commit/bea4f68875104c0f589aa818a362f6f871f1e97b))
* **m1-global:** register tauri-plugin-store and tauri-plugin-global-shortcut in builder ([e7721e2](https://github.com/matheusmarnt/orcker/commit/e7721e2fb7269745a9da0e817d3f9e4b5fbafe6b))
* **m2-projects:** 3-stage start with Docker restart loop for failed containers ([546d1c6](https://github.com/matheusmarnt/orcker/commit/546d1c6b15c2d37b83f41e0dd357c52e39f1d10e))
* **m2-projects:** add EnvEditor component with diff view against .env.example ([4cc2831](https://github.com/matheusmarnt/orcker/commit/4cc2831a05c453f219e7f108ad4ac26ce149cb62))
* **m2-projects:** add loading spinners and descriptive text to project action buttons ([827cb52](https://github.com/matheusmarnt/orcker/commit/827cb52d38bf4faa6df8b236db0977c31b42b980))
* **m2-projects:** add php.ini parser and supervisor Rust commands ([9b9266e](https://github.com/matheusmarnt/orcker/commit/9b9266e6b11d1a0ac31e7a8df2eeb349f1bd5501))
* **m2-projects:** add project and compose type contracts ([3a5dbc7](https://github.com/matheusmarnt/orcker/commit/3a5dbc71b11a12cab7eedc30a5de645eb0468094))
* **m2-projects:** add project Tauri commands with import detection ([5ece7fa](https://github.com/matheusmarnt/orcker/commit/5ece7fab737e8524bcc71564368e698b64b71a39))
* **m2-projects:** add ProjectCard, NewProjectModal, and ProjectsView ([501cf12](https://github.com/matheusmarnt/orcker/commit/501cf12b48a067cd7c412053ebd2944832c8ed27))
* **m2-projects:** add ProjectDetailView, /projects/:id route, and Terminal nav ([5aaaca7](https://github.com/matheusmarnt/orcker/commit/5aaaca76409958c5648ec1a4804163fc57b36b2a))
* **m2-projects:** add read_env_file and save_env_file Rust commands ([c647f7b](https://github.com/matheusmarnt/orcker/commit/c647f7baa28d3111aebe247b28779b150a01eb0e))
* **m2-projects:** add scaffold_project command with Channel&lt;T&gt; streaming ([2183b6d](https://github.com/matheusmarnt/orcker/commit/2183b6deff104b364f8a9f7558758f8790b75c3a))
* **m2-projects:** add start/stop/open project commands with Sail auto-detection ([1c69d10](https://github.com/matheusmarnt/orcker/commit/1c69d10bd24cff9b063dddbfe8587c77e4f2a092))
* **m2-projects:** add toggle_vite_auto and generate_xdebug_config commands ([dbc3ea2](https://github.com/matheusmarnt/orcker/commit/dbc3ea225a15a2fbc8955fe98538b56787ffd0ae))
* **m2-projects:** implement useProjectsStore with init, register, import, pickFolder ([f443986](https://github.com/matheusmarnt/orcker/commit/f4439860e816d87fca46020c0e63fe7c0639695b))
* **m2-projects:** poll container health after compose up before returning success ([d9a6a91](https://github.com/matheusmarnt/orcker/commit/d9a6a919e0212a00298dd75cb611a461c92e40b1))
* **m2-projects:** register php.ini/supervisor commands and add Vue components ([85e9634](https://github.com/matheusmarnt/orcker/commit/85e9634c455994f928bc19dd516c35e4ebffb001))
* **m2-projects:** register scaffold_project, regen bindings, wire Scaffold tab ([a1665ce](https://github.com/matheusmarnt/orcker/commit/a1665ce26dd692d820df4bfc0b5f9ec0a7c7a5f8))
* **m2-projects:** register toggle_vite_auto and generate_xdebug_config in lib.rs, regen bindings ([fa9fa2e](https://github.com/matheusmarnt/orcker/commit/fa9fa2e70b3426076e7cd65845406bd549b63504))
* **m2-projects:** retry app container start when healthcheck deps fail ([f350318](https://github.com/matheusmarnt/orcker/commit/f350318c11efa708f456840f26fa9b6ee7f313cd))
* **m2-projects:** strict start policy + live project status monitor ([cae9912](https://github.com/matheusmarnt/orcker/commit/cae991270f0b079889a757193d1b34c426e8f6b3))
* **m2-projects:** wire project commands into lib.rs and regen bindings ([859a22c](https://github.com/matheusmarnt/orcker/commit/859a22c05cff39f7973f0fb00c12a5f053e83434))
* **m3-terminal:** add artisan command catalog ([9fec50b](https://github.com/matheusmarnt/orcker/commit/9fec50b9db2fe8bb525b6840dee6dc4f4229ea3f))
* **m3-terminal:** add CommandPanel and DestructiveConfirmDialog components ([fe465b6](https://github.com/matheusmarnt/orcker/commit/fe465b6604c2d32ffd43c2939ff3ededc591a5a0))
* **m3-terminal:** add docker exec streaming adapter with CancellationToken ([567bfd1](https://github.com/matheusmarnt/orcker/commit/567bfd1c7867c253bad5314fd5d050eb603b061f))
* **m3-terminal:** wire artisan commands into lib.rs and regen bindings ([aa5da30](https://github.com/matheusmarnt/orcker/commit/aa5da30c3f4c983ed2872f9b40401b0c4dd708a7))
* **m4-logs:** add log_stream adapter with CancellationToken and ring buffer support ([6aacd55](https://github.com/matheusmarnt/orcker/commit/6aacd55463b43248d072a22653153b70caa5c527))
* **m4-logs:** add LogViewer, LogLine, LogFilterBar, LogsView ([5ab86d7](https://github.com/matheusmarnt/orcker/commit/5ab86d709b594aaa1acac203d3173319eba3588f))
* **m4-logs:** add start_log_stream and stop_log_stream Tauri commands ([15ef687](https://github.com/matheusmarnt/orcker/commit/15ef687e7174777773ee0cc9c7d01f2c778fe9ed))
* **m4-logs:** implement useLogsStore with ring buffer and filtering ([c538224](https://github.com/matheusmarnt/orcker/commit/c538224f6781e6fc748d81dbc4afe452c1a7a487))
* **rust:** scaffold Phase 1 Rust modules with bollard, AppState and AppError ([bed6a6c](https://github.com/matheusmarnt/orcker/commit/bed6a6ccde0e7c2d82c2b163342134a5adfea93f))
* **rust:** wire tauri-specta Builder, tracing and probe_docker in lib.rs ([12b86e4](https://github.com/matheusmarnt/orcker/commit/12b86e422e07b89ea2ef55fc63022242609b8d41))
* **ui:** add collapsible AppSidebar layout with persistent state ([0aa92cb](https://github.com/matheusmarnt/orcker/commit/0aa92cb02926e112e33a9c47e3ecec989742a625))
* **ui:** add ContainerTable, DockerStatusBadge and ErrorScreen ([22df46f](https://github.com/matheusmarnt/orcker/commit/22df46faf9d25d38020fc4d974884ef100712b02))
* **ui:** bootstrap Vue app with Pinia, router and typed IPC layer ([f0d1d27](https://github.com/matheusmarnt/orcker/commit/f0d1d2708bdd205eb0b6283496613a8058cb04a9))
* **ui:** install frontend deps and initialize shadcn-vue with reka-ui ([11818ae](https://github.com/matheusmarnt/orcker/commit/11818ae2e8a1f049936f3a373e179a88d43472a6))
* **ui:** scaffold Tauri 2 + Vue 3 + TypeScript base app ([bf9e5cc](https://github.com/matheusmarnt/orcker/commit/bf9e5ccf4307150a604c92af2612e8858447243a))
* **ui:** wire DashboardView with skeleton, table and ErrorScreen via Pinia store ([1b3763c](https://github.com/matheusmarnt/orcker/commit/1b3763cf0bc6d0b9cdbb1315af95da698bef55d4))


### Bug Fixes

* **ci:** correct release-please version strategy to stay pre-1.0.0 ([114e4dc](https://github.com/matheusmarnt/orcker/commit/114e4dc6f3a98ba57250639269721524d4a45f16))
* **ci:** pass vitest when no test files exist yet ([4259675](https://github.com/matheusmarnt/orcker/commit/4259675049c31289ea347386071f8e5800ee0acb))
* **ci:** replace single-arm match with if let in start_project ([dd77e60](https://github.com/matheusmarnt/orcker/commit/dd77e60b1f8a5707c8a3a07536391c294a4522bf))
* **ci:** resolve eslint errors in vite-env.d.ts and approve esbuild build scripts ([4255737](https://github.com/matheusmarnt/orcker/commit/4255737e857387968cf1cc14ffba7819c9ddea94))
* **ci:** resolve TS type check and Rust fmt failures ([0842d47](https://github.com/matheusmarnt/orcker/commit/0842d4758bebe880032056193d3096e5135c73e7))
* **ci:** set initial-version 0.1.0 to prevent release-please proposing 1.0.0 ([28cc5e4](https://github.com/matheusmarnt/orcker/commit/28cc5e454e3357adb43462ecdb63667b42e108c9))
* **m1-global:** auto-pull Docker image before creating container and add error logging ([f0068b8](https://github.com/matheusmarnt/orcker/commit/f0068b89121607d0c21ac5f8e38097ff2f11da76))
* **m1-global:** fix PostgreSQL container startup with env vars and fail-fast polling ([fa30880](https://github.com/matheusmarnt/orcker/commit/fa30880db85c5fde22fb3fb35cfbbb045086358d))
* **m1-global:** fix Switch API for reka-ui v2 and flex alignment of cards ([9422dfc](https://github.com/matheusmarnt/orcker/commit/9422dfc6d2eae93cacaa2c3cec16487ebabaf83a))
* **m1-global:** fix toast system — CSS import, Vue context and state transitions ([0df931f](https://github.com/matheusmarnt/orcker/commit/0df931fc92901009767ea2fade57a2a3dd33d6df))
* **m2-projects,m3-terminal:** remove fallback container and trim compose error ([abd781e](https://github.com/matheusmarnt/orcker/commit/abd781e4d8ac41691f4a88da1d0643a2cb439b0b))
* **m2-projects,m4-logs:** auto-detect app container via Docker labels, add error handling ([fb45702](https://github.com/matheusmarnt/orcker/commit/fb45702723eb82e0861dbbc968a0a7ad293a8331))
* **m2-projects,m4-logs:** wire start/stop buttons and add logs loading state ([1d31698](https://github.com/matheusmarnt/orcker/commit/1d31698614c048ca85bc590e70b9cfe9e32222bf))
* **m2-projects:** catch re-thrown IPC errors in start/stop/open handlers ([5661cf5](https://github.com/matheusmarnt/orcker/commit/5661cf5b6ed66ffb710c05a353fe7d725702df3b))
* **m3-terminal,m4-logs:** handle typedError status result and show last 100KB of log file ([c50042c](https://github.com/matheusmarnt/orcker/commit/c50042c00557938c2beed3189d8e6aec05b68965))
* **m4-logs:** suppress bollard 0.19 deprecation warnings in log_stream ([190a47a](https://github.com/matheusmarnt/orcker/commit/190a47af6ce750bb8c1138959d34f36aa8997010))
* **rust:** add 500ms delay before first probe to sync Vue listeners ([9ac0c7c](https://github.com/matheusmarnt/orcker/commit/9ac0c7c463be6ed3283ae1319f9a68072ddaa408))
* **rust:** collapse nested if let in stop_service (clippy::collapsible_match) ([488c4a8](https://github.com/matheusmarnt/orcker/commit/488c4a8f172b2d3cf0078de9e210b003eeb32e51))
* **rust:** remove fn pointer coercion for async fn in network compilation test ([6a19130](https://github.com/matheusmarnt/orcker/commit/6a19130d77edce99013183a6601f36a532a03007))
* **rust:** remove tauri_plugin_log — conflicts with tracing_log::LogTracer ([33603ad](https://github.com/matheusmarnt/orcker/commit/33603ad4646e1b4edb1a7bd66e741c0b989ab9eb))
* **rust:** use try_init in tracing_subscriber to avoid conflict with tauri_plugin_log ([c1479be](https://github.com/matheusmarnt/orcker/commit/c1479be8d8cd1cb5d92998832433daf802f0dbe3))
* **ui:** add global Toaster and fix config panel with v-if ([9f46b6d](https://github.com/matheusmarnt/orcker/commit/9f46b6dd9c6170ededa6befdc91c9ebb7bdd6567))
* **ui:** move listener init to App setup and add state fallback ([f135c47](https://github.com/matheusmarnt/orcker/commit/f135c472086b8aedb04f54828c3cb7ae90037aa3))

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
