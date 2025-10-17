use std::usize;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "fviewr",
    help_template = "\
{about-section}
使用方式: {usage}

参数：
{positionals}

选项：
{options}
{after-help}"
)]
pub struct Args {
    /// 期望查看的目标文件夹
    #[arg(default_value_t = String::from("."), value_name = "目标路径")]
    pub target: String,
    /// 期望展示的文件层级深度
    #[arg(short, long, default_value_t = 1, value_name = "嵌套深度")]
    pub deep: usize,
    /// 展示隐藏的文件
    #[arg(short, long, default_value_t = false, value_name = "查看隐藏文件")]
    pub all: bool
}
