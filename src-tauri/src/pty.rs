use parking_lot::Mutex;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

struct PtySession {
    writer: Box<dyn Write + Send>,
    _master: Box<dyn MasterPty + Send>,
}

pub struct PtyManager {
    sessions: Mutex<HashMap<String, PtySession>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    pub fn start(
        &self,
        app: AppHandle,
        terminal_node_id: String,
        cols: u16,
        rows: u16,
        cwd: Option<String>,
        context_file: Option<String>,
        bash_rcfile: Option<String>,
    ) -> Result<(), String> {
        {
            let sessions = self.sessions.lock();
            if sessions.contains_key(&terminal_node_id) {
                return Ok(());
            }
        }

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| e.to_string())?;

        // Prefer bash + session rcfile so cursor-agent is wrapped with --workspace
        let (program, args): (String, Vec<String>) = if let Some(rc) = bash_rcfile {
            (
                "/bin/bash".to_string(),
                vec!["--rcfile".to_string(), rc, "-i".to_string()],
            )
        } else {
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
            (shell, vec!["-i".to_string()])
        };

        let mut cmd = CommandBuilder::new(&program);
        for a in args {
            cmd.arg(a);
        }
        cmd.env("TERM", "xterm-256color");
        if let Some(hint) = context_file {
            cmd.env("CONTEXT_MAP_FILE", hint);
        }
        if let Some(dir) = &cwd {
            cmd.env("CONTEXT_MAP_SESSION", dir);
            cmd.cwd(dir);
        }

        let mut child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| e.to_string())?;
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| e.to_string())?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| e.to_string())?;

        let event_name = format!("pty-output-{}", terminal_node_id);
        let exit_event = format!("pty-exit-{}", terminal_node_id);
        let id_for_reader = terminal_node_id.clone();
        let app_reader = app.clone();

        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buf[..n]).to_string();
                        let _ = app_reader.emit(&event_name, data);
                    }
                    Err(_) => break,
                }
            }
            let _ = child.wait();
            let _ = app_reader.emit(&exit_event, id_for_reader);
        });

        let session = PtySession {
            writer,
            _master: pair.master,
        };

        self.sessions.lock().insert(terminal_node_id, session);
        Ok(())
    }

    pub fn write(&self, terminal_node_id: &str, data: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock();
        let session = sessions
            .get_mut(terminal_node_id)
            .ok_or_else(|| "Terminal session not running".to_string())?;
        session
            .writer
            .write_all(data.as_bytes())
            .map_err(|e| e.to_string())?;
        session.writer.flush().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn resize(&self, terminal_node_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let sessions = self.sessions.lock();
        let session = sessions
            .get(terminal_node_id)
            .ok_or_else(|| "Terminal session not running".to_string())?;
        session
            ._master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| e.to_string())
    }

    pub fn stop(&self, terminal_node_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock();
        sessions.remove(terminal_node_id);
        Ok(())
    }

    pub fn is_running(&self, terminal_node_id: &str) -> bool {
        self.sessions.lock().contains_key(terminal_node_id)
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedPty = Arc<PtyManager>;
