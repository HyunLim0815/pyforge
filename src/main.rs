//! PyForge 二进制入口。
//!
//! 初始化 tracing → 解析 CLI → 分发命令。

mod agent;
mod cli;
#[allow(dead_code)]
mod index;
mod i18n;
mod output;
mod system;
mod uv;
mod templates;
mod web;

use clap::Parser;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = cli::Cli::parse();

    // 初始化外部语言包，再设置语言
    i18n::init_external();
    i18n::init_lang(cli.lang.as_deref().unwrap_or(""));

    if let Err(code) = cli::dispatch(cli).await {
        std::process::exit(code);
    }
}
