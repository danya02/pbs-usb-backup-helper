use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// The URL to the source code file defining the config format.
    #[serde(rename = "_readme_schema")]
    pub help_schema_url: String,

    /// If true, this filesystem will be considered for backups.
    pub do_backup: bool,

    /// The PBS repository URL.
    /// See https://pbs.proxmox.com/docs/backup-client.html#backup-repository-locations
    pub repository: String,

    /// The change detection mode to use.
    /// See https://pbs.proxmox.com/docs/backup-client.html#change-detection-mode
    pub change_detection_mode: ChangeDetectionMode,

    /// The namespace to store the data in.
    /// If empty, the root namespace will be used.
    pub namespace: String,

    /// The crypt-mode to use.
    /// Currently only "none" is supported.
    pub crypt_mode: CryptMode,

    /// The backup ID.
    /// This is approximately similar to a host name:
    /// the backup group will be named "host/backup_id".
    pub backup_id: String,

    /// The name of the main backup archive.
    /// The contents of this volume will be stored as "{pxar_name}.pxar" in PBS.
    pub pxar_name: String,

    /// The type of backup.
    /// The utility of this is not yet clear:
    /// it definitely affects the icon type in PBS to be used,
    /// and is part of the backup group's name.
    pub backup_type: BackupType,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            help_schema_url:
                "https://github.com/danya02/pbs-usb-backup-helper/blob/main/src/config.rs".into(),
            do_backup: false,
            repository: "pbs.example.internal:datastore".into(),
            change_detection_mode: ChangeDetectionMode::Metadata,
            namespace: String::new(),
            crypt_mode: CryptMode::None,
            backup_id: "random_flash_drive".into(),
            pxar_name: "flash_drive".into(),
            backup_type: BackupType::Host,
        }
    }
}

/// https://pbs.proxmox.com/docs/backup-client.html#change-detection-mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeDetectionMode {
    #[serde(rename = "legacy")]
    Legacy,
    #[serde(rename = "data")]
    Data,
    #[serde(rename = "metadata")]
    Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CryptMode {
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupType {
    #[serde(rename = "vm")]
    VirtualMachine,
    #[serde(rename = "ct")]
    Container,
    #[serde(rename = "host")]
    Host,
}

pub async fn get_or_create_config(mountpoint: &str) -> anyhow::Result<BackupConfig> {
    let file_path = format!("{mountpoint}/ProxmoxBackupConfig.toml");

    let text = match tokio::fs::read_to_string(&file_path).await {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("No config file found, creating a new one at {file_path}");
            let config = BackupConfig::default();
            let toml = toml::to_string(&config).unwrap();
            tokio::fs::write(&file_path, toml).await?;
            return Ok(config);
        }
        Err(e) => {
            println!("Failed to read config file {file_path}: {e}");
            return Err(e.into());
        }
    };

    let config: BackupConfig = toml::from_str(&text)?;
    Ok(config)
}
