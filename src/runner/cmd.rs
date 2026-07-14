use crate::{
    config::{RunCommand, Task},
    runner::{log_buffer::LogBuffer, types::TaskState},
};
use std::{
    collections::HashMap,
    process::Stdio,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

pub async fn run_shell_command(
    task_name: &str,
    states: &Arc<Mutex<HashMap<String, TaskState>>>,
    cmd_str: &str,
    working_dir: &Option<std::path::PathBuf>,
    output_buf: &Arc<Mutex<LogBuffer>>,
    prefix: &Option<String>,
    is_tui: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut command = Command::new("sh");
    command.arg("-c").arg(cmd_str);
    if let Some(dir) = working_dir {
        command.current_dir(dir);
    }
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    if is_tui {
        command.stdin(Stdio::piped());
    } else {
        command.stdin(Stdio::inherit());
    }

    #[cfg(unix)]
    {
        command.process_group(0);
    }

    let mut child = command.spawn()?;
    let pid = child.id();

    let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    // Set child_pid and stdin_tx in states
    {
        let mut guard = states.lock().unwrap();
        if let Some(state) = guard.get_mut(task_name) {
            if let Some(pid) = pid {
                state.child_pid = Some(pid);
            }
            if is_tui {
                state.stdin_tx = Some(stdin_tx);
            }
        }
    }

    struct PidGuard {
        task_name: String,
        states: Arc<Mutex<HashMap<String, TaskState>>>,
    }

    impl Drop for PidGuard {
        fn drop(&mut self) {
            let mut guard = self.states.lock().unwrap();
            if let Some(state) = guard.get_mut(&self.task_name) {
                state.child_pid = None;
                state.stdin_tx = None;
            }
        }
    }

    let _pid_guard = PidGuard {
        task_name: task_name.to_string(),
        states: Arc::clone(states),
    };

    let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to open stderr")?;
    let child_stdin = child.stdin.take();

    let stdin_handle = if let Some(mut child_in) = child_stdin {
        Some(tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            while let Some(bytes) = stdin_rx.recv().await {
                if child_in.write_all(&bytes).await.is_err() {
                    break;
                }
                let _ = child_in.flush().await;
            }
        }))
    } else {
        None
    };

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
    if let Some(handle) = stdin_handle {
        handle.abort();
    }

    let status = child.wait().await?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Command exited with status: {}", status).into())
    }
}

pub async fn execute_command_capturing(
    task_name: &str,
    states: &Arc<Mutex<HashMap<String, TaskState>>>,
    task: &Task,
    output_buf: &Arc<Mutex<LogBuffer>>,
    prefix: &Option<String>,
    is_tui: bool,
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
            run_shell_command(
                task_name,
                states,
                cmd_str,
                &task.working_dir,
                output_buf,
                prefix,
                is_tui,
            )
            .await
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
                run_shell_command(
                    task_name,
                    states,
                    cmd_str,
                    &task.working_dir,
                    output_buf,
                    prefix,
                    is_tui,
                )
                .await?;
            }
            Ok(())
        }
    }
}
