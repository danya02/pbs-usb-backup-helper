use zvariant::OwnedObjectPath;

use crate::{config, fs_mount_and_init};

pub struct BackupJob {
    pub object_path: OwnedObjectPath,
    pub cancel: tokio::sync::oneshot::Sender<()>,
    pub join_handle: tokio::task::JoinHandle<()>,
}

pub async fn run_backup_for_object(obj_path: OwnedObjectPath) -> anyhow::Result<BackupJob> {
    let mountpoint = fs_mount_and_init::mount(obj_path.as_ref()).await?;
    let config = config::get_or_create_config(&mountpoint).await?;

    if !config.do_backup {
        return Err(anyhow::anyhow!(
            "Backup is disabled for {mountpoint}. Enable it in the config file."
        ));
    }

    let (cancel_send, cancel_recv) = tokio::sync::oneshot::channel();

    let join_handle = tokio::spawn(async move {
        run_backup(&mountpoint, cancel_recv).await.unwrap();
    });

    let job = BackupJob {
        cancel: cancel_send,
        join_handle,
        object_path: obj_path,
    };

    Ok(job)
}

async fn run_backup(
    mountpoint: &str,
    mut cancel: tokio::sync::oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    println!("TODO: Starting backup for {mountpoint}");

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                println!("TODO: Backup for {mountpoint} still running");
            }
            _ = &mut cancel => {
                println!("TODO: Backup for {mountpoint} cancelled");
                break;
            }
        }
    }

    Ok(())
}
