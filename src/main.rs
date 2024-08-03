use futures_util::TryStreamExt;
use std::collections::HashMap;

use zbus::{message::Type, Connection, MessageStream};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    let connection = Connection::system().await?;

    let mut props = HashMap::<String, zvariant::Value>::new();
    props.insert("auth.no_user_interaction".into(), true.into());

    let resp = connection
        .call_method(
            Some("org.freedesktop.UDisks2"),
            "/org/freedesktop/UDisks2/Manager",
            Some("org.freedesktop.UDisks2.Manager"),
            "GetBlockDevices",
            // (a{sv})
            &(props,),
        )
        .await?;
    println!("Got response: {resp:#?}");
    let outp: Vec<zvariant::OwnedObjectPath> = resp.body().deserialize()?;

    println!("Got response: {outp:#?}");

    connection
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus.Monitoring"),
            "BecomeMonitor",
            &(&["sender=org.freedesktop.UDisks2"] as &[&str], 0u32),
        )
        .await?;

    let mut stream = MessageStream::from(connection);
    while let Some(msg) = stream.try_next().await? {
        if let Type::Signal = msg.message_type() {
            println!("Got signal: {msg} {msg:?}");
            if let Some(sig) = msg.body().signature() {
                let body = msg.body();
                msg.header().member();
                match sig.as_str() {
                    "sa{sv}as" => {
                        let data: (String, HashMap<String, zvariant::Value>, Vec<String>) =
                            body.deserialize()?;
                        println!("{data:#?}");
                    }
                    other => println!("Unknown signature: {other}"),
                }
            }
        }
    }

    // let properties_proxy = zbus::fdo::PropertiesProxy::builder(&connection)
    //     .destination("org.freedesktop.UDisks2")?
    //     .path("/org/freedesktop/UDisks2/drives")?
    //     .build()
    //     .await?;

    // let mut changes_stream = properties_proxy.receive_properties_changed().await?;

    // while let Some(change) = changes_stream.try_next().await {
    //     println!("Change body: {change:#?}");
    // }

    // connection
    //     .call_method(
    //         Some("org.freedesktop.DBus"),
    //         "/org/freedesktop/DBus",
    //         Some("org.freedesktop.DBus.Monitoring"),
    //         "BecomeMonitor",
    //         &(&[] as &[&str], 0u32),
    //     )
    //     .await?;

    // let mut stream = MessageStream::from(connection);
    // while let Some(msg) = stream.try_next().await? {
    //     println!("Got message: {}", msg);
    // }

    Ok(())
}
