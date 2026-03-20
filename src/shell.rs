use anyhow::Result;
use clap::Parser;
use rustyline::DefaultEditor;

use crate::Cli;
use crate::output;

/// 交互式 REPL
pub async fn run_shell() -> Result<()> {
    let mut rl = DefaultEditor::new()?;

    // 尝试加载历史
    let history_path = crate::config::config_path()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("history")));
    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    println!("DEX interactive shell. Type 'help' for commands, 'exit' to quit.");

    loop {
        let line = match rl.readline("dex> ") {
            Ok(line) => line,
            Err(rustyline::error::ReadlineError::Interrupted | rustyline::error::ReadlineError::Eof) => break,
            Err(e) => return Err(e.into()),
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let _ = rl.add_history_entry(trimmed);

        match trimmed {
            "exit" | "quit" => break,
            "shell" => {
                eprintln!("Cannot nest shell sessions.");
                continue;
            }
            "setup" => {
                eprintln!("Run 'dex setup' outside the shell.");
                continue;
            }
            _ => {}
        }

        // 解析命令: 前缀 "dex" + 用户输入
        let args = std::iter::once("dex".to_string())
            .chain(split_args(trimmed))
            .collect::<Vec<_>>();

        match Cli::try_parse_from(&args) {
            Ok(cli) => {
                if let Err(e) = crate::run(cli).await {
                    output::print_error(&e, output::OutputFormat::Table);
                }
            }
            Err(e) => {
                let _ = e.print();
            }
        }
    }

    // 保存历史
    if let Some(ref path) = history_path {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = rl.save_history(path);
    }

    Ok(())
}

/// 拆分参数，支持引号
fn split_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = ' ';

    for ch in input.chars() {
        if in_quote {
            if ch == quote_char {
                in_quote = false;
            } else {
                current.push(ch);
            }
        } else if ch == '"' || ch == '\'' {
            in_quote = true;
            quote_char = ch;
        } else if ch.is_whitespace() {
            if !current.is_empty() {
                args.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        args.push(current);
    }

    args
}
