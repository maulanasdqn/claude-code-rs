use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use stynx_code_errors::{AppError, AppResult};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::Mutex;
use tokio::sync::mpsc;

const MAX_BG_OUTPUT: usize = 1_000_000;
const READ_CHUNK: usize = 8192;

pub struct ShellRegistry {
    persistent: Mutex<Option<PersistentShell>>,
    background: Mutex<HashMap<String, BgProcess>>,
    next_bg_id: Mutex<u64>,
}

struct PersistentShell {
    stdin: ChildStdin,
    out_rx: mpsc::UnboundedReceiver<Vec<u8>>,
    child: Child,
    leftover: Vec<u8>,
}

struct BgProcess {
    cmd: String,
    started_at: Instant,
    output: Arc<Mutex<Vec<u8>>>,
    exit: Arc<Mutex<Option<i32>>>,
    child: Arc<Mutex<Option<Child>>>,
    last_read_pos: Arc<Mutex<usize>>,
}

impl ShellRegistry {
    pub fn new() -> Self {
        Self {
            persistent: Mutex::new(None),
            background: Mutex::new(HashMap::new()),
            next_bg_id: Mutex::new(1),
        }
    }

    pub async fn run_sync(&self, command: &str, timeout: Option<Duration>) -> AppResult<String> {
        if let Some(reason) = detect_interactive(command) {
            return Err(AppError::Tool(format!(
                "{reason}\n\
                The persistent shell has no TTY — interactive prompts will deadlock and corrupt the TUI. \
                Options: (a) use non-interactive flags (ssh: -o BatchMode=yes with key auth; sudo: -n with passwordless sudoers); \
                (b) pre-share the secret via env / config; (c) suggest the user runs `! <command>` themselves to handle the prompt interactively."
            )));
        }
        let mut guard = self.persistent.lock().await;
        let need_spawn = match guard.as_mut() {
            None => true,
            Some(s) => s.is_dead(),
        };
        if need_spawn {
            *guard = Some(PersistentShell::spawn().await?);
        }
        let result = guard.as_mut().unwrap().run(command, timeout).await;
        if let Err(AppError::Tool(ref msg)) = result {
            if msg.starts_with("command timed out") {
                tracing::warn!(command = %command, "bash command timed out, resetting persistent shell");
                *guard = None;
            }
        }
        result
    }

    pub async fn run_background(&self, command: &str) -> AppResult<String> {
        let handle = {
            let mut id_guard = self.next_bg_id.lock().await;
            let id = *id_guard;
            *id_guard += 1;
            format!("bg{id}")
        };

        let mut child = Command::new("bash")
            .arg("-c")
            .arg(command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::Tool(format!("bg spawn failed: {e}")))?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let output = Arc::new(Mutex::new(Vec::<u8>::new()));
        let exit = Arc::new(Mutex::new(None::<i32>));
        let child_arc = Arc::new(Mutex::new(Some(child)));

        spawn_bg_reader(stdout, output.clone());
        spawn_bg_reader(stderr, output.clone());

        let exit_clone = exit.clone();
        let child_clone = child_arc.clone();
        tokio::spawn(async move {
            let code = {
                let mut g = child_clone.lock().await;
                match g.as_mut() {
                    Some(c) => c.wait().await.ok().and_then(|s| s.code()),
                    None => None,
                }
            };
            *exit_clone.lock().await = code;
        });

        let bg = BgProcess {
            cmd: command.to_string(),
            started_at: Instant::now(),
            output,
            exit,
            child: child_arc,
            last_read_pos: Arc::new(Mutex::new(0)),
        };
        self.background.lock().await.insert(handle.clone(), bg);
        Ok(handle)
    }

    pub async fn read_background(&self, handle: &str, full: bool) -> AppResult<String> {
        let guard = self.background.lock().await;
        let bg = guard
            .get(handle)
            .ok_or_else(|| AppError::Tool(format!("no background process '{handle}'")))?;
        let output = bg.output.lock().await;
        let exit = *bg.exit.lock().await;
        let mut pos_guard = bg.last_read_pos.lock().await;
        let start = if full { 0 } else { (*pos_guard).min(output.len()) };
        let text = String::from_utf8_lossy(&output[start..]).into_owned();
        *pos_guard = output.len();

        let elapsed = bg.started_at.elapsed().as_secs();
        let status = match exit {
            Some(c) if c == 0 => format!("[exit 0, ran {elapsed}s]"),
            Some(c) => format!("[exit {c}, ran {elapsed}s]"),
            None => format!("[running, {elapsed}s]"),
        };
        if text.is_empty() {
            Ok(format!("{status}  (no new output)"))
        } else {
            Ok(format!("{status}\n{text}"))
        }
    }

    pub async fn kill_background(&self, handle: &str) -> AppResult<String> {
        let guard = self.background.lock().await;
        let bg = guard
            .get(handle)
            .ok_or_else(|| AppError::Tool(format!("no background process '{handle}'")))?;
        let mut child_guard = bg.child.lock().await;
        if let Some(c) = child_guard.as_mut() {
            let _ = c.start_kill();
        }
        Ok(format!("killed {handle}"))
    }

    pub async fn list_background(&self) -> String {
        let guard = self.background.lock().await;
        if guard.is_empty() {
            return "no background processes".into();
        }
        let mut out = String::from("background processes:\n");
        for (k, v) in guard.iter() {
            let exit = *v.exit.lock().await;
            let elapsed = v.started_at.elapsed().as_secs();
            let state = match exit {
                Some(c) => format!("exit {c}"),
                None => "running".into(),
            };
            let cmd = first_line(&v.cmd);
            let cmd = if cmd.len() > 80 { format!("{}…", &cmd[..79]) } else { cmd.to_string() };
            out.push_str(&format!("  {k:<6} [{state:<10}] {elapsed}s  $ {cmd}\n"));
        }
        out
    }
}

fn spawn_bg_reader<R>(mut r: R, output: Arc<Mutex<Vec<u8>>>)
where
    R: AsyncReadExt + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut buf = vec![0u8; READ_CHUNK];
        loop {
            match r.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let mut g = output.lock().await;
                    g.extend_from_slice(&buf[..n]);
                    if g.len() > MAX_BG_OUTPUT {
                        let excess = g.len() - MAX_BG_OUTPUT;
                        g.drain(..excess);
                    }
                }
                Err(_) => break,
            }
        }
    });
}

impl PersistentShell {
    async fn spawn() -> AppResult<Self> {
        let mut child = Command::new("bash")
            .args(["--norc", "--noprofile"])
            .env("PS1", "")
            .env("PS2", "")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| AppError::Tool(format!("failed to spawn persistent shell: {e}")))?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let (tx, out_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        Self::spawn_reader(stdout, tx.clone());
        Self::spawn_reader(stderr, tx);

        Ok(Self { stdin, out_rx, child, leftover: Vec::new() })
    }

    fn spawn_reader<R>(mut r: R, tx: mpsc::UnboundedSender<Vec<u8>>)
    where
        R: AsyncReadExt + Unpin + Send + 'static,
    {
        tokio::spawn(async move {
            let mut buf = vec![0u8; READ_CHUNK];
            loop {
                match r.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            return;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    fn is_dead(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(Some(_)))
    }

    async fn run(&mut self, command: &str, timeout: Option<Duration>) -> AppResult<String> {
        let nonce = nonce_hex();
        let marker = format!("__STYNX_DONE_{nonce}__");

        let payload = format!(
            "{{\n{command}\n}} 2>&1\nprintf '\\n%s:%d\\n' '{marker}' $?\n"
        );
        self.stdin
            .write_all(payload.as_bytes())
            .await
            .map_err(|e| AppError::Tool(format!("shell write failed: {e}")))?;
        self.stdin.flush().await.ok();

        let needle = format!("\n{marker}:");
        let needle_bytes = needle.as_bytes();

        let mut buf = std::mem::take(&mut self.leftover);
        let deadline = timeout.map(|d| Instant::now() + d);

        loop {
            while let Ok(chunk) = self.out_rx.try_recv() {
                buf.extend_from_slice(&chunk);
            }

            if let Some(pos) = find_subslice(&buf, needle_bytes) {
                let after = pos + needle_bytes.len();
                if let Some(nl_rel) = buf[after..].iter().position(|&b| b == b'\n') {
                    let nl_abs = after + nl_rel;
                    let exit_slice = &buf[after..nl_abs];
                    let exit_code = std::str::from_utf8(exit_slice)
                        .ok()
                        .and_then(|s| s.parse::<i32>().ok())
                        .unwrap_or(-1);
                    let output_bytes = &buf[..pos];
                    let text = String::from_utf8_lossy(output_bytes).into_owned();
                    self.leftover = buf[nl_abs + 1..].to_vec();
                    if exit_code == 0 {
                        return Ok(if text.is_empty() { "(no output)".into() } else { text });
                    }
                    return Ok(format!("{text}\n[exit {exit_code}]"));
                }
            }

            if let Some(d) = deadline {
                if Instant::now() >= d {
                    self.leftover = buf;
                    let secs = timeout.map(|t| t.as_secs()).unwrap_or(0);
                    return Err(AppError::Tool(format!(
                        "command timed out after {secs}s. \
the persistent shell is now in an unknown state — consider running \
the command with background:true if it's a long-running process."
                    )));
                }
            }

            match tokio::time::timeout(Duration::from_millis(40), self.out_rx.recv()).await {
                Ok(Some(chunk)) => buf.extend_from_slice(&chunk),
                Ok(None) => {
                    self.leftover = buf;
                    return Err(AppError::Tool(
                        "persistent shell stdout closed unexpectedly".into(),
                    ));
                }
                Err(_) => {}
            }
        }
    }
}

fn detect_interactive(command: &str) -> Option<String> {
    let trimmed = command.trim();
    let head = trimmed.split('|').next().unwrap_or("").trim();
    let mut tokens = head.split_whitespace();
    let Some(first) = tokens.next() else { return None; };
    let rest: Vec<&str> = tokens.collect();

    match first {
        "ssh" => {
            let has_batchmode = rest.windows(2).any(|w| w[0] == "-o" && w[1].eq_ignore_ascii_case("BatchMode=yes"));
            let has_i_arg = rest.iter().any(|t| *t == "-i" || t.starts_with("-i") && t.len() > 2);
            if !has_batchmode && !has_i_arg {
                return Some("blocked: `ssh` without key auth or `-o BatchMode=yes` will hit a password prompt.".into());
            }
        }
        "scp" | "sftp" | "rsync" => {
            let has_batchmode = rest.windows(2).any(|w| w[0] == "-o" && w[1].eq_ignore_ascii_case("BatchMode=yes"));
            if !has_batchmode && !rest.iter().any(|t| *t == "-i") {
                return Some(format!("blocked: `{first}` will prompt for a password without key auth or `-o BatchMode=yes`."));
            }
        }
        "sudo" => {
            let has_n = rest.iter().any(|t| *t == "-n" || *t == "--non-interactive");
            if !has_n {
                return Some("blocked: `sudo` without `-n` will prompt for a password.".into());
            }
        }
        "passwd" | "su" | "login" => {
            return Some(format!("blocked: `{first}` is interactive and cannot run in the persistent shell."));
        }
        "vim" | "vi" | "nvim" | "nano" | "emacs" | "less" | "more" | "top" | "htop" | "btop" | "watch" | "tmux" | "screen" | "man" => {
            return Some(format!("blocked: `{first}` is a TUI/curses program and will deadlock the persistent shell."));
        }
        "mysql" | "psql" | "sqlite3" | "redis-cli" | "mongo" | "mongosh" => {
            let has_command_flag = rest.iter().any(|t| matches!(*t, "-c" | "--command" | "-e" | "--execute"));
            if !has_command_flag {
                return Some(format!("blocked: `{first}` without `-c`/`-e` opens an interactive REPL that will deadlock."));
            }
        }
        "gpg" | "ssh-keygen" | "ssh-add" => {
            let has_batch = rest.iter().any(|t| *t == "--batch" || *t == "--pinentry-mode=loopback" || *t == "-N");
            if !has_batch {
                return Some(format!("blocked: `{first}` may prompt for a passphrase — pass `--batch` or `-N \"\"` if you know what you're doing."));
            }
        }
        _ => {}
    }
    None
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

fn nonce_hex() -> String {

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id() as u128;
    format!("{:032x}", now ^ (pid << 96))
}

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s)
}
