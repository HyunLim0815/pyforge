use super::Cli;
use clap::Args;

#[derive(Args, Debug)]
pub struct WebArgs {
    /// 监听端口
    #[arg(long, default_value_t = 7742)]
    pub port: u16,
    /// 禁止自动打开浏览器
    #[arg(long, default_value_t = false)]
    pub no_open: bool,
}

pub async fn handle(_cli: &Cli, args: &WebArgs) -> Result<(), i32> {
    crate::web::server::run(args.port, !args.no_open)
        .await
        .map_err(|e| {
            eprintln!("web server error: {}", e);
            2
        })
}
