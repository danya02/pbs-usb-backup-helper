use std::collections::HashMap;

use backup::BackupJob;
use fs_detect::FsEvent;
use zvariant::OwnedObjectPath;

mod backup;
mod config;
mod fs_detect;
mod fs_mount_and_init;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");

    let (send, mut recv) = tokio::sync::mpsc::channel(10);

    tokio::spawn(async move {
        fs_detect::loop_detect_filesystems(send).await.unwrap();
    });

    let (create_send, create_recv) = tokio::sync::mpsc::channel(10);
    let (delete_send, delete_recv) = tokio::sync::mpsc::channel(10);
    tokio::spawn(job_handler(create_recv, delete_recv));

    while let Some(event) = recv.recv().await {
        println!("Event: {event:?}");
        if let FsEvent::FilesystemAdded(obj) = event {
            tokio::spawn({
                let create_send = create_send.clone();
                async move {
                    let job = backup::run_backup_for_object(obj).await.unwrap();
                    create_send.send(job).await.unwrap();
                }
            });
        } else if let FsEvent::FilesystemRemoved(obj) = event {
            println!("Filesystem removed: {obj:?}");
            delete_send.send(obj).await.unwrap();
        }
    }

    Ok(())
}

async fn job_handler(
    mut create_recv: tokio::sync::mpsc::Receiver<BackupJob>,
    mut delete_recv: tokio::sync::mpsc::Receiver<OwnedObjectPath>,
) {
    let mut jobs = HashMap::new();
    loop {
        tokio::select! {
            Some(job) = create_recv.recv() => {
                jobs.insert(job.object_path.clone(), job);
            }
            Some(obj) = delete_recv.recv() => {
                println!("Filesystem removed: {obj:?}");
                if let Some(job) = jobs.remove(&obj) {
                    println!("Stopping backup for {obj:?}");
                    job.cancel.send(()).unwrap();
                    job.join_handle.await.unwrap();
                }
            }
        }
    }
}
