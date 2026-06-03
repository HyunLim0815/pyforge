//! Shell 补全脚本生成。
//!
//! 使用 clap_complete 生成 bash/zsh/fish/powershell 补全脚本。

use clap::Command;
use clap_complete::{generate, Shell};
use std::io;

/// 输出指定 shell 的补全脚本到 stdout。
pub fn generate_completion(shell: Shell, cmd: &mut Command) {
    let name = cmd.get_name().to_string();
    generate(shell, cmd, &name, &mut io::stdout());
}
