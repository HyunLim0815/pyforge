//! English message corpus.

pub fn get(key: &str) -> Option<String> {
    let msg = match key {
        // test
        "test.hello" => "Hello".to_string(),

        // track
        "track.success" => "Project registered: {0}".to_string(),
        "track.already" => "Project already tracked: {0}".to_string(),
        "track.path_not_found" => "Path not found: {0}".to_string(),

        // list
        "list.empty" => "No tracked projects".to_string(),
        "list.item" => "{0}  {1}".to_string(),

        // scan
        "scan.found" => "Found {0} Python projects".to_string(),
        "scan.progress" => "Scanning {0} directories...".to_string(),
        "scan.no_match" => "No Python projects found".to_string(),

        // new
        "new.success" => "Project created: {0}".to_string(),
        "new.dir_exists" => "Directory already exists: {0}".to_string(),
        "new.uv_missing" => "uv not found. Install: https://docs.astral.sh/uv/".to_string(),

        // mkpkg
        "mkpkg.created" => "Subpackage created: {0}".to_string(),
        "mkpkg.skipped" => "Skipped (exists): {0}".to_string(),
        "mkpkg.no_name" => "Specify at least one package name".to_string(),
        "mkpkg.dry_run" => "[dry-run] Would create: {0}".to_string(),

        // info / untrack / goto
        "info.not_found" => "Project not in index: {0}".to_string(),
        "untrack.success" => "Removed from index: {0}".to_string(),
        "untrack.not_found" => "Project not in index: {0}".to_string(),

        // status
        "status.empty" => "No tracked projects".to_string(),

        // outdated
        "outdated.all_up_to_date" => "All dependencies up to date".to_string(),
        "outdated.uv_missing" => "uv not found. Install uv first.".to_string(),
        "outdated.parse_error" => "Failed to parse dependency information".to_string(),
        "outdated.item" => "  {0}: {1} -> {2}".to_string(),

        // agent
        "agent.skill_written" => "SKILL.md written: {0}".to_string(),

        // template
        "template.unknown" => "Unknown template: {0}. Available templates: {1}".to_string(),
        "template.custom_shadowed" => {
            "Custom template '{0}' shadows a built-in template, using built-in".to_string()
        }

        // i18n
        "i18n.list.header" => "Available languages:".to_string(),
        "i18n.list.builtin" => "builtin".to_string(),
        "i18n.list.external" => "installed".to_string(),
        "i18n.list.none" => "No external language packs installed".to_string(),
        "i18n.install.success" => "Language pack installed: {0}".to_string(),
        "i18n.install.already" => "Language pack already exists: {0}".to_string(),
        "i18n.install.fail" => "Installation failed: {0}".to_string(),
        "i18n.install.invalid" => "Invalid language pack format, not installed".to_string(),

        // CLI generic
        "cli.not_implemented" => "Command '{0}' not yet implemented".to_string(),

        _ => return None,
    };
    Some(msg)
}
