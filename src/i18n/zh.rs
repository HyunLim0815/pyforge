//! 中文消息语料。

pub fn get(key: &str) -> Option<String> {
    let msg = match key {
        // test
        "test.hello" => "你好".to_string(),

        // track
        "track.success" => "已注册项目: {0}".to_string(),
        "track.already" => "项目已在索引中: {0}".to_string(),
        "track.path_not_found" => "路径不存在: {0}".to_string(),

        // list
        "list.empty" => "暂无已跟踪的项目".to_string(),
        "list.item" => "{0}  {1}".to_string(),

        // scan
        "scan.found" => "发现 {0} 个 Python 项目".to_string(),
        "scan.progress" => "正在扫描 {0} 个目录...".to_string(),
        "scan.no_match" => "未发现 Python 项目".to_string(),

        // new
        "new.success" => "项目已创建: {0}".to_string(),
        "new.dir_exists" => "目录已存在: {0}".to_string(),
        "new.uv_missing" => "未找到 uv，请先安装: https://docs.astral.sh/uv/".to_string(),

        // mkpkg
        "mkpkg.created" => "已创建子包: {0}".to_string(),
        "mkpkg.skipped" => "已跳过（存在）: {0}".to_string(),
        "mkpkg.no_name" => "请指定至少一个包名".to_string(),
        "mkpkg.dry_run" => "[dry-run] 将创建: {0}".to_string(),

        // info / untrack / goto
        "info.not_found" => "项目不在索引中: {0}".to_string(),
        "untrack.success" => "已从索引移除: {0}".to_string(),
        "untrack.not_found" => "项目不在索引中: {0}".to_string(),

        // status
        "status.empty" => "暂无已跟踪的项目".to_string(),

        // outdated
        "outdated.all_up_to_date" => "所有依赖为最新".to_string(),
        "outdated.uv_missing" => "未找到 uv，请先安装".to_string(),
        "outdated.parse_error" => "无法解析依赖信息".to_string(),
        "outdated.item" => "  {0}: {1} -> {2}".to_string(),

        // agent
        "agent.skill_written" => "SKILL.md 已写入: {0}".to_string(),

        // template
        "template.unknown" => "未知模板: {0}。可用模板: {1}".to_string(),
        "template.custom_shadowed" => "自定义模板 '{0}' 与内置模板同名，使用内置模板".to_string(),

        // i18n
        "i18n.list.header" => "可用语言:".to_string(),
        "i18n.list.builtin" => "内置".to_string(),
        "i18n.list.external" => "已安装".to_string(),
        "i18n.list.none" => "无已安装的外部语言包".to_string(),
        "i18n.install.success" => "语言包已安装: {0}".to_string(),
        "i18n.install.already" => "语言包已存在: {0}".to_string(),
        "i18n.install.fail" => "安装失败: {0}".to_string(),
        "i18n.install.invalid" => "语言包格式无效，未安装".to_string(),

        // CLI generic
        "cli.not_implemented" => "'{0}' 命令尚未实现".to_string(),

        _ => return None,
    };
    Some(msg)
}
