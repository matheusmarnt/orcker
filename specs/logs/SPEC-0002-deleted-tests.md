# SPEC-0002 — deleted-test review list

Mechanically derived: every `#[test]`/`#[tokio::test]` fn present at `HEAD` and
absent from the SPEC-0002 diff, **restricted to files that still exist**.

- 671 test fns deleted in total
- 445 died with their file (crate/module removed by R1/R2 - authorized)
- 90 in `wire_stability.rs` (R4-authorized reset, verified byte-identical)
- **136 deleted from surviving files** - the entire DT7 surface

Reproduce with `/tmp/enum_tests.py` (see cycle log S7 attempt 3).

| status | file | test | note |
| --- | --- | --- | --- |
| drop | `apps/orcker-gui/src-tauri/src/tray.rs` | `default_php_choices_drops_legacy_versions` | subject `default_php_choices` deleted by R5; unrestorable |
| drop | `apps/orcker-gui/src-tauri/src/tray.rs` | `default_php_choices_is_empty_when_only_legacy_is_installed` | subject `default_php_choices` deleted by R5; unrestorable |
| [ ] | `bin/orcker/src/cli.rs` | `exec_captures_hyphenated_tool_args` |
| [ ] | `bin/orcker/src/cli.rs` | `exec_forwards_a_trailing_json_flag_to_the_tool` |
| [ ] | `bin/orcker/src/cli.rs` | `exec_rejects_an_unknown_tool` |
| [ ] | `bin/orcker/src/cli.rs` | `exec_takes_orckers_json_flag_before_the_tool` |
| [ ] | `bin/orcker/src/cli.rs` | `exec_takes_site_before_the_tool` |
| [ ] | `bin/orcker/src/cli.rs` | `which_parses_php_with_an_optional_site` |
| [ ] | `bin/orcker/src/cli.rs` | `which_rejects_composer` |
| [ ] | `bin/orcker/src/lib.rs` | `canonicalize_db_paths_backup_absolute_is_unchanged` |
| [ ] | `bin/orcker/src/lib.rs` | `canonicalize_db_paths_backup_relative_is_absolutised` |
| [ ] | `bin/orcker/src/lib.rs` | `canonicalize_db_paths_other_request_passes_through` |
| [ ] | `bin/orcker/src/lib.rs` | `canonicalize_db_paths_restore_existing_file_is_canonicalised` |
| [ ] | `bin/orcker/src/lib.rs` | `canonicalize_db_paths_restore_missing_file_is_usage_error` |
| [ ] | `bin/orcker/src/lib.rs` | `first_php_on_path_returns_option` |
| [ ] | `bin/orcker/src/lib.rs` | `print_php_path_hint_runs` |
| [ ] | `bin/orcker/src/map.rs` | `install_legacy_requires_flag` |
| [ ] | `bin/orcker/src/map.rs` | `install_php_rejects_bad_version` |
| [ ] | `bin/orcker/src/map.rs` | `maps_each_command_to_its_request` |
| [ ] | `bin/orcker/src/map.rs` | `maps_every_db_action` |
| [ ] | `bin/orcker/src/map.rs` | `maps_every_service_action` |
| [ ] | `bin/orcker/src/map.rs` | `maps_service_set_and_unset_to_a_single_entry_override_map` |
| [ ] | `bin/orcker/src/map.rs` | `maps_services_command` |
| [ ] | `bin/orcker/src/map.rs` | `php_ext_add_maps_and_defaults_name` |
| [ ] | `bin/orcker/src/map.rs` | `php_ext_add_rejects_non_absolute_path_client_side` |
| [ ] | `bin/orcker/src/map.rs` | `php_ext_list_and_remove_map` |
| [ ] | `bin/orcker/src/map.rs` | `php_ini_actions_map_and_validate` |
| [ ] | `bin/orcker/src/map.rs` | `php_pool_actions_map_and_validate` |
| [ ] | `bin/orcker/src/map.rs` | `rejects_bad_version_and_name_before_connect` |
| [ ] | `bin/orcker/src/map.rs` | `renders_available_php_legacy_section` |
| [ ] | `bin/orcker/src/map.rs` | `renders_available_php_tagging_installed` |
| [ ] | `bin/orcker/src/map.rs` | `renders_available_services` |
| [ ] | `bin/orcker/src/map.rs` | `renders_databases` |
| [ ] | `bin/orcker/src/map.rs` | `renders_empty_php_extensions` |
| [ ] | `bin/orcker/src/map.rs` | `renders_human_responses_and_exit_codes` |
| [ ] | `bin/orcker/src/map.rs` | `renders_php_extensions_grouped_with_missing_flag` |
| [ ] | `bin/orcker/src/map.rs` | `renders_php_settings_block` |
| [ ] | `bin/orcker/src/map.rs` | `renders_php_versions_marking_default` |
| [ ] | `bin/orcker/src/map.rs` | `renders_php_versions_with_per_version_overrides_and_directives` |
| [ ] | `bin/orcker/src/map.rs` | `renders_service_logs` |
| [ ] | `bin/orcker/src/map.rs` | `renders_service_overrides` |
| [ ] | `bin/orcker/src/map.rs` | `renders_services_table_and_states` |
| [ ] | `bin/orcker/src/map.rs` | `service_set_refuses_a_bad_shape_and_a_service_without_overrides` |
| [ ] | `bin/orcker/src/map.rs` | `service_set_refuses_a_reserved_key_in_either_spelling` |
| [ ] | `bin/orcker/src/map.rs` | `unset_unknown_php_setting_is_usage_error` |
| [ ] | `bin/orcker/src/map.rs` | `update_php_with_self_update_flags_each_error` |
| [ ] | `bin/orcker/src/map.rs` | `use_rejects_legacy_default` |
| [ ] | `bin/orcker/tests/cli_e2e.rs` | `php_version_config_round_trips_against_daemon` |
| [ ] | `bin/orckerd/src/backend_resolver.rs` | `explicit_override_beats_the_detected_default` |
| [ ] | `bin/orckerd/src/backend_resolver.rs` | `subdir_framework_funnels_but_root_served_executes_directly` |
| [ ] | `bin/orckerd/src/backend_resolver.rs` | `wordpress_in_subdir_still_executes_directly` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `add_php_extension_invalid_path_rejected_before_probe` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `add_php_extension_uninstalled_version_is_not_found` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `adopt_default_if_unset_never_adopts_legacy` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `available_php_errors_on_fetch_failure` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `available_php_lists_distribution_minors_and_installed` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `available_php_lists_legacy_from_second_manifest` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `bundle_contains_ca_empty_ca_is_never_trusted` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `bundle_contains_ca_false_when_absent` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `bundle_contains_ca_matches_embedded_ca` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `check_update_falls_back_to_cache_when_offline` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `dispatch_diagnose_flags_a_reserved_key_in_the_local_override_file` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `dispatch_diagnose_flags_missing_php` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `dispatch_doctor_fix_rebuilds_missing_php_ca_bundle` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `dispatch_doctor_fix_with_no_pools_is_noop_but_reports_manual` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `dispatch_dumps_status_lifecycle` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `dispatch_list_php_no_update_when_cache_not_newer` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `dispatch_list_php_reports_installed_and_default` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `dispatch_list_php_surfaces_cached_update` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `dispatch_list_php_surfaces_revision_autoheal` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `dispatch_list_services_reports_all_engines_uninstalled` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `dispatch_restart_all_php_no_pools_is_ok` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `dispatch_set_default_php_requires_installed` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `dispatch_set_default_php_sets_config_and_shim` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `dispatch_update_php_unknown_is_not_found` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `install_php_legacy_without_confirm_is_rejected` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `install_php_streamed_legacy_without_confirm_creates_no_job` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `installed_versions_empty_then_lists_fake_install` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `legacy_install_gate_blocks_only_unconfirmed_legacy` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `map_pool_state_maps_both_variants` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `poll_and_refresh_checks_legacy_when_stable_is_unreachable` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `poll_and_refresh_is_channel_aware_for_legacy` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `poll_and_refresh_is_failure_tolerant` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `poll_and_refresh_keeps_cache_on_unknown_schema` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `poll_and_refresh_populates_cache_from_listing` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `poll_and_refresh_preserves_cached_legacy_update_when_legacy_fetch_fails` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `poll_and_refresh_tolerates_untrusted_legacy_manifest` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `pool_settings_are_refused_through_the_directives_path` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `pools_needing_restart_only_targets_active_updated_minors` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `remove_and_list_php_extensions` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `restart_php_not_installed_is_not_found` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `set_default_php_changes_fallback_for_new_sites` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `set_default_php_rejects_legacy_even_when_installed` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `set_php_directives_persists_rejects_reserved_and_removes` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `set_php_pool_settings_persists_validates_and_removes` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `set_php_settings_persists_validates_and_resets` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `set_php_version_settings_persists_canonicalises_and_falls_back` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `site_needing_url_sync_covers_all_domain_mutations` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `site_needing_url_sync_finds_mixed_case_name` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `site_needing_url_sync_none_for_non_domain_requests` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `uninstall_php_blocked_when_default_with_others` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `uninstall_php_blocked_when_in_use_by_site` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `uninstall_php_not_installed_is_not_found` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `uninstall_php_succeeds_and_removes_dir` |
| [ ] | `bin/orckerd/src/ipc_server.rs` | `use_overrides_parked_site_keeping_kind_mixed_case` |
| [ ] | `bin/orckerd/src/mutate.rs` | `set_php_on_a_proxy_name_says_it_is_a_proxy` |
| [ ] | `bin/orckerd/src/mutate.rs` | `set_wordpress_auto_login_records_override_keeping_parked` |
| [ ] | `bin/orckerd/src/mutate.rs` | `set_wordpress_auto_login_unknown_is_not_found` |
| [ ] | `bin/orckerd/src/mutate.rs` | `set_wordpress_auto_login_updates_linked_in_place` |
| [ ] | `bin/orckerd/src/mutate.rs` | `setphp_records_override_keeping_parked` |
| [ ] | `bin/orckerd/src/mutate.rs` | `setphp_unknown_is_not_found` |
| [ ] | `bin/orckerd/src/mutate.rs` | `setphp_updates_linked_in_place` |
| [ ] | `bin/orckerd/src/mutate.rs` | `upsert_merges_php_and_secure` |
| [ ] | `bin/orckerd/src/startup.rs` | `build_php_ca_bundle_no_roots_does_not_reuse_ca_only_or_stale_bundle` |
| [ ] | `bin/orckerd/src/startup.rs` | `build_php_ca_bundle_no_roots_reuses_existing_good_bundle_unchanged` |
| [ ] | `bin/orckerd/src/startup.rs` | `build_php_ca_bundle_with_rootless_content_returns_none_and_does_not_write` |
| [ ] | `bin/orckerd/src/startup.rs` | `build_php_ca_bundle_with_roots_returns_path_and_writes_bundle` |
| [ ] | `bin/orckerd/src/startup.rs` | `build_php_ca_bundle_without_roots_returns_none_and_does_not_write` |
| [ ] | `bin/orckerd/src/tools/mod.rs` | `move_dir_contents_moves_all_entries` |
| [ ] | `bin/orckerd/src/tools/mod.rs` | `wp_cli_never_accepts_an_external_copy` |
| [ ] | `crates/orcker-doctor/src/lib.rs` | `a_clean_local_override_file_produces_no_finding` |
| [ ] | `crates/orcker-doctor/src/lib.rs` | `a_garbage_line_warns_and_names_the_line` |
| [ ] | `crates/orcker-doctor/src/lib.rs` | `a_reserved_key_warns_and_carries_its_hint` |
| [ ] | `crates/orcker-doctor/src/lib.rs` | `a_service_without_a_dialect_is_skipped` |
| [ ] | `crates/orcker-doctor/src/lib.rs` | `an_override_finding_suppresses_the_all_good_line` |
| [ ] | `crates/orcker-doctor/src/lib.rs` | `default_not_installed_when_other_versions_present` |
| [ ] | `crates/orcker-doctor/src/lib.rs` | `failed_pool_is_fail_and_auto_fixable` |
| [ ] | `crates/orcker-doctor/src/lib.rs` | `no_php_suppresses_default_not_installed` |
| [ ] | `crates/orcker-doctor/src/lib.rs` | `php_ca_none_or_true_emits_no_finding` |
| [ ] | `crates/orcker-doctor/src/lib.rs` | `php_ca_untrusted_warns_and_plans_rebuild` |
| [ ] | `crates/orcker-doctor/src/lib.rs` | `update_available_is_informational_and_still_all_good` |
| [ ] | `crates/orcker-mcp/tests/tools.rs` | `create_site_defaults_match_the_gui_wizard` |
| [ ] | `crates/orcker-mcp/tests/tools.rs` | `create_site_options_are_mapped` |
| [ ] | `crates/orcker-mcp/tests/tools.rs` | `optional_arguments_reach_the_request` |
| [ ] | `crates/orcker-mcp/tests/tools.rs` | `php_tools_map_to_their_requests` |
| [ ] | `crates/orcker-mcp/tests/tools.rs` | `read_tools_map_to_their_requests` |
