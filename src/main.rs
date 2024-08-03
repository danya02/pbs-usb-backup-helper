use fs_detect::FsEvent;

mod fs_detect;
mod fs_mount_and_init;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");

    let (send, mut recv) = tokio::sync::mpsc::channel(10);

    tokio::spawn(async move {
        fs_detect::loop_detect_filesystems(send).await.unwrap();
    });

    while let Some(event) = recv.recv().await {
        println!("Event: {event:?}");
        if let FsEvent::FilesystemAdded(obj) = event {
            fs_mount_and_init::mount_and_init(obj).await.unwrap();
        }
    }

    Ok(())
}
