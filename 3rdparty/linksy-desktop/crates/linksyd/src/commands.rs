//! Preset saved terminal commands for paired phones.
//! - The computer shares its commands list with connected devices.
//! - A connected phone can execute a command by its key.
//! - Auto-imports existing commands from KDE Connect configuration.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use linksy_core::protocol::CommandEntry;
use linksy_core::{Event, Fingerprint, Node};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::error::RecvError;
use tracing::{info, warn};

#[derive(Clone, Serialize, Deserialize)]
pub struct Commands {
    inner: Arc<Mutex<Vec<CommandEntry>>>,
    path: PathBuf,
}

impl Commands {
    pub fn load(path: &Path) -> Self {
        let file_path = path.join("commands.json");
        let list = if let Ok(data) = std::fs::read(&file_path) {
            serde_json::from_slice(&data).unwrap_or_else(|_| Self::default_commands())
        } else {
            let imported = Self::try_import_kdeconnect();
            let commands = if !imported.is_empty() {
                imported
            } else {
                Self::default_commands()
            };
            if let Ok(json) = serde_json::to_vec_pretty(&commands) {
                let _ = std::fs::create_dir_all(path);
                let _ = std::fs::write(&file_path, json);
            }
            commands
        };

        Self {
            inner: Arc::new(Mutex::new(list)),
            path: file_path,
        }
    }

    pub fn list(&self) -> Vec<CommandEntry> {
        self.inner.lock().unwrap().clone()
    }

    pub fn add(&self, name: String, command: String) {
        let mut guard = self.inner.lock().unwrap();
        let key = Self::next_key(&guard);
        guard.push(CommandEntry { key, name, command });
        drop(guard);
        let _ = self.save();
    }

    pub fn edit(&self, key: &str, name: String, command: String) -> bool {
        let mut guard = self.inner.lock().unwrap();
        if let Some(entry) = guard.iter_mut().find(|c| c.key == key) {
            entry.name = name;
            entry.command = command;
            drop(guard);
            let _ = self.save();
            true
        } else {
            false
        }
    }

    pub fn remove(&self, key: &str) -> bool {
        let mut guard = self.inner.lock().unwrap();
        let before = guard.len();
        guard.retain(|c| c.key != key);
        if guard.len() == before {
            return false;
        }
        drop(guard);
        let _ = self.save();
        true
    }

    fn next_key(existing: &[CommandEntry]) -> String {
        let n = existing.iter().filter_map(|c| {
            c.key.strip_prefix("cmd-").and_then(|s| s.parse::<u64>().ok())
        }).max().unwrap_or(0);
        format!("cmd-{}", n + 1)
    }

    fn save(&self) -> std::io::Result<()> {
        let list = self.list();
        let json = serde_json::to_vec_pretty(&list).unwrap_or_default();
        std::fs::create_dir_all(self.path.parent().unwrap_or(&self.path))?;
        std::fs::write(&self.path, json)
    }

    pub async fn run(self, node: Node) {
        let mut events = node.subscribe();
        loop {
            match events.recv().await {
                Ok(Event::Connected { device }) => {
                    let list = self.list();
                    let _ = node.send_commands(&device.id, list).await;
                }
                Ok(Event::RequestCommandsReceived { from }) => {
                    let list = self.list();
                    let _ = node.send_commands(&from.id, list).await;
                }
                Ok(Event::RunCommandRequested { from, key }) => {
                    self.execute(&node, from.id, &key).await;
                }
                Ok(_) | Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => break,
            }
        }
    }

    async fn execute(&self, node: &Node, from: Fingerprint, key: &str) {
        let is_trusted = node.devices().into_iter().any(|d| d.device.id == from && d.trusted);
        if !is_trusted {
            warn!("untrusted device {from} attempted to run command {key}");
            return;
        }

        let entry = {
            let guard = self.inner.lock().unwrap();
            guard.iter().find(|c| c.key == key).cloned()
        };

        if let Some(c) = entry {
            info!("executing command '{}': {}", c.name, c.command);
            let mut cmd = tokio::process::Command::new("/bin/sh");
            cmd.args(["-c", &c.command]);
            if let Err(e) = cmd.spawn() {
                warn!("failed to run command '{}': {e}", c.name);
            }
        } else {
            warn!("unknown command key requested: {key}");
        }
    }

    fn default_commands() -> Vec<CommandEntry> {
        vec![
            CommandEntry {
                key: "lock-session".into(),
                name: "Lock Screen".into(),
                command: "loginctl lock-session".into(),
            },
            CommandEntry {
                key: "panic-button".into(),
                name: "Panic Button".into(),
                command: "ps -eo pid,%mem,comm --sort=-%mem | head -n 2 | tail -n 1 | awk '{print $1}' | xargs kill -9".into(),
            },
            CommandEntry {
                key: "restart-plasma".into(),
                name: "Restart Plasma".into(),
                command: "systemctl --user restart plasma-plasmashell.service".into(),
            },
            CommandEntry {
                key: "turn-off-screen".into(),
                name: "Turn Off Screen".into(),
                command: "kscreen-doctor --dpms off".into(),
            },
            CommandEntry {
                key: "suspend".into(),
                name: "Suspend".into(),
                command: "systemctl suspend".into(),
            },
            CommandEntry {
                key: "reboot".into(),
                name: "Reboot".into(),
                command: "reboot".into(),
            },
            CommandEntry {
                key: "shutdown".into(),
                name: "Shutdown".into(),
                command: "shutdown now".into(),
            },
        ]
    }

    fn try_import_kdeconnect() -> Vec<CommandEntry> {
        let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        let config_dir = home.join(".config/kdeconnect");
        if !config_dir.exists() {
            return Vec::new();
        }

        let mut results = Vec::new();
        if let Ok(entries) = std::fs::read_dir(config_dir) {
            for entry in entries.flatten() {
                let conf_file = entry.path().join("kdeconnect_runcommand/config");
                if conf_file.exists() {
                    if let Ok(content) = std::fs::read_to_string(&conf_file) {
                        for line in content.lines() {
                            if let Some(json_start) = line.find("@ByteArray(") {
                                let raw = &line[json_start + 11..];
                                if let Some(end) = raw.rfind(')') {
                                    let json_str = &raw[..end];
                                    if let Ok(parsed) = serde_json::from_str::<HashMap<String, HashMap<String, String>>>(json_str) {
                                        for (k, v) in parsed {
                                            if let (Some(name), Some(command)) = (v.get("name"), v.get("command")) {
                                                if !results.iter().any(|c: &CommandEntry| c.key == k) {
                                                    results.push(CommandEntry {
                                                        key: k,
                                                        name: name.clone(),
                                                        command: command.clone(),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        results
    }
}
