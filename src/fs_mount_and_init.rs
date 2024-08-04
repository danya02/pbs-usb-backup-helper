use std::collections::HashMap;

use zbus::Connection;
use zvariant::ObjectPath;

pub async fn mount(path: ObjectPath<'_>) -> anyhow::Result<String> {
    println!("Mounting and initializing: {path:?}");
    let connection = Connection::system().await?;

    let mut options = HashMap::<String, zvariant::Value>::new();
    options.insert("auth.no_user_interaction".into(), true.into());
    let result = connection
        .call_method(
            Some("org.freedesktop.UDisks2"),
            &path,
            Some("org.freedesktop.UDisks2.Filesystem"),
            "Mount",
            &(&options),
        )
        .await?;

    println!("Got response: {result:?}");
    let mountpoint: String = result.body().deserialize()?;

    println!("Got mountpoint: {mountpoint:?}");

    Ok(mountpoint)
}
