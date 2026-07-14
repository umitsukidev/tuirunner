use crate::config::{RunCommand, Task};
use std::{
    process::Stdio,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

pub async fn run_shell_command(
    cmd_str: &str,
    working_dir: &Option<std::path::PathBuf>,
    output_buf: &Arc<Mutex<Vec<String>>>,
    prefix: &Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut command = Command::new("sh");
    command.arg("-c").arg(cmd_str);
    if let Some(dir) = working_dir {
        command.current_dir(dir);
    }
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn()?;

    let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to open stderr")?;

    let output_buf_stdout = Arc::clone(output_buf);
    let prefix_stdout = prefix.clone();
    let stdout_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout);
        let mut buf = Vec::new();
        while let Ok(n) = reader.read_until(b'\n', &mut buf).await {
            if n == 0 {
                break;
            }
            let mut line = String::from_utf8_lossy(&buf).into_owned();
            if line.ends_with('\n') {
                line.pop();
            }
            if line.ends_with('\r') {
                line.pop();
            }
            if let Some(ref pref) = prefix_stdout {
                println!("{} {}", pref, line);
            }
            let mut output = output_buf_stdout.lock().unwrap();
            output.push(line);
            buf.clear();
        }
    });

    let output_buf_stderr = Arc::clone(output_buf);
    let prefix_stderr = prefix.clone();
    let stderr_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr);
        let mut buf = Vec::new();
        while let Ok(n) = reader.read_until(b'\n', &mut buf).await {
            if n == 0 {
                break;
            }
            let mut line = String::from_utf8_lossy(&buf).into_owned();
            if line.ends_with('\n') {
                line.pop();
            }
            if line.ends_with('\r') {
                line.pop();
            }
            if let Some(ref pref) = prefix_stderr {
                eprintln!("{} {}", pref, line);
            }
            let mut output = output_buf_stderr.lock().unwrap();
            output.push(line);
            buf.clear();
        }
    });

    let _ = tokio::join!(stdout_handle, stderr_handle);

    let status = child.wait().await?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Command exited with status: {}", status).into())
    }
}

pub async fn execute_command_capturing(
    task: &Task,
    output_buf: &Arc<Mutex<Vec<String>>>,
    prefix: &Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let run_cmd = if let Some(ref r) = task.run {
        Some((r, false))
    } else if let Some(ref c) = task.cmd {
        Some((c, true))
    } else {
        None
    };

    match run_cmd {
        None => Ok(()),
        Some((RunCommand::Single(cmd_str), show_command)) => {
            if show_command {
                use crossterm::style::Stylize;
                let styled_cmd = format!("$ {}", cmd_str).dim();
                if let Some(pref) = prefix {
                    println!("{} {}", pref, styled_cmd);
                }
                {
                    let mut buf = output_buf.lock().unwrap();
                    buf.push(styled_cmd.to_string());
                }
            }
            run_shell_command(cmd_str, &task.working_dir, output_buf, prefix).await
        }
        Some((RunCommand::Multiple(cmds), show_command)) => {
            for cmd_str in cmds {
                if show_command {
                    use crossterm::style::Stylize;
                    let styled_cmd = format!("$ {}", cmd_str).dim();
                    if let Some(pref) = prefix {
                        println!("{} {}", pref, styled_cmd);
                    }
                    {
                        let mut buf = output_buf.lock().unwrap();
                        buf.push(styled_cmd.to_string());
                    }
                }
                run_shell_command(cmd_str, &task.working_dir, output_buf, prefix).await?;
            }
            Ok(())
        }
    }
}
