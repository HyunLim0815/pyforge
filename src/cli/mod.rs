//! CLI 命令定义（clap derive）。
//!
//! [Source: architecture.md §Process Patterns]
//! - 每个命令 handler 入口打开 `info_span!("cmd::{name}")`
//! - stdout 承载数据，stderr 承载诊断输出
//! - 退出码：0 成功，1 用户错误，2 系统错误

mod agent_info;
mod completion;
mod goto;
mod i18n;
mod info;
mod init_agent;
mod list;
mod mkpkg;
mod new;
mod outdated;
mod scan;
mod status;
mod track;
mod untrack;
mod web;

use crate::output::OutputFormat;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

/// PyForge — Python 项目管家，uv 的强力补充。
#[derive(Parser, Debug)]
#[command(name = "pyforge", version, about)]
pub struct Cli {
    /// 以 JSON 格式输出（stdout 仅输出 JSON）
    #[arg(long, global = true, default_value_t = false)]
    pub json: bool,

    /// 输出语言（zh / en），优先级：--lang > PYFORGE_LANG > 中文
    #[arg(long, global = true)]
    pub lang: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    #[allow(dead_code)]
    pub fn output_format(&self) -> OutputFormat {
        if self.json {
            OutputFormat::Json
        } else {
            OutputFormat::Plain
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 注册项目到本地索引
    #[command(name = "track")]
    Track(track::TrackArgs),
    /// 从索引中移除项目
    #[command(name = "untrack")]
    Untrack(untrack::UntrackArgs),
    /// 列出所有已跟踪项目
    #[command(name = "list")]
    List,
    /// 扫描目录并批量注册 Python 项目
    #[command(name = "scan")]
    Scan(scan::ScanArgs),
    /// 查看项目详细信息
    #[command(name = "info")]
    Info(info::InfoArgs),
    /// 创建新项目（封装 uv init + 自动注册）
    #[command(name = "new")]
    New(new::NewArgs),
    /// 在当前项目中批量创建子包
    #[command(name = "mkpkg")]
    Mkpkg(mkpkg::MkpkgArgs),
    /// 输出项目路径（配合 shell 跳转）
    #[command(name = "goto")]
    Goto(goto::GotoArgs),
    /// 显示所有项目状态概览
    #[command(name = "status")]
    Status(status::StatusArgs),
    /// 启动 Web 管理看板
    #[command(name = "web")]
    Web(web::WebArgs),
    /// 输出 AI Agent 环境信息
    #[command(name = "agent-info")]
    AgentInfo,
    /// 安装 AI Agent 技能文件
    #[command(name = "init-agent")]
    InitAgent(init_agent::InitAgentArgs),
    /// 生成 Shell 补全脚本
    Completion {
        /// Shell 类型
        #[arg(value_enum)]
        shell: Shell,
    },
    /// 检查项目依赖更新状态
    #[command(name = "outdated")]
    Outdated(outdated::OutdatedArgs),
    /// 语言包管理
    #[command(name = "i18n")]
    I18n(i18n::I18nArgs),
}

/// 路由子命令到对应的 handler。
pub async fn dispatch(cli: Cli) -> Result<(), i32> {
    let _span = tracing::info_span!("cmd", name = cli.command.name()).entered();
    match &cli.command {
        Commands::Track(args) => track::handle(&cli, args),
        Commands::List => list::handle(&cli),
        Commands::Scan(args) => scan::handle(&cli, args),
        Commands::Info(args) => info::handle(&cli, args),
        Commands::Untrack(args) => untrack::handle(&cli, args),
        Commands::Goto(args) => goto::handle(&cli, args),
        Commands::New(args) => new::handle(&cli, args),
        Commands::Mkpkg(args) => mkpkg::handle(&cli, args),
        Commands::Status(args) => status::handle(&cli, args).await,
        Commands::Completion { shell } => {
            let mut cmd = Cli::command();
            completion::generate_completion(*shell, &mut cmd);
            Ok(())
        }
        Commands::Outdated(args) => outdated::handle(&cli, args),
        Commands::AgentInfo => agent_info::handle(&cli),
        Commands::I18n(args) => i18n::handle(&cli, args),
        Commands::Web(args) => web::handle(&cli, args).await,
        Commands::InitAgent(args) => init_agent::handle(&cli, args),
    }
}

/// 返回命令名称。
impl Commands {
    pub fn name(&self) -> &'static str {
        match self {
            Commands::Track(_) => "track",
            Commands::Untrack(_) => "untrack",
            Commands::List => "list",
            Commands::Scan(_) => "scan",
            Commands::Info(_) => "info",
            Commands::New(_) => "new",
            Commands::Mkpkg(_) => "mkpkg",
            Commands::Goto(_) => "goto",
            Commands::Status(_) => "status",
            Commands::Web(_) => "web",
            Commands::AgentInfo => "agent-info",
            Commands::InitAgent(_) => "init-agent",
            Commands::Completion { .. } => "completion",
            Commands::Outdated(_) => "outdated",
            Commands::I18n(_) => "i18n",
        }
    }
}
