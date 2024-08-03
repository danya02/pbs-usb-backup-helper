use futures_util::TryStreamExt;
use std::collections::HashMap;

use zbus::{message::Type, names::MemberName, Connection, MessageStream};

#[derive(Debug)]
pub enum FsEvent {
    FilesystemAdded(zvariant::OwnedObjectPath),
    FilesystemRemoved(zvariant::OwnedObjectPath),
}

pub async fn loop_detect_filesystems(
    send: tokio::sync::mpsc::Sender<FsEvent>,
) -> anyhow::Result<()> {
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
                match msg
                    .header()
                    .member()
                    .unwrap_or(&MemberName::from_static_str_unchecked("__empty__"))
                    .as_str()
                {
                    "PropertiesChanged" => {
                        let (interface, new_property_values, _): (
                            String,
                            HashMap<String, zvariant::Value>,
                            Vec<String>,
                        ) = body.deserialize()?;
                        println!("PropertiesChanged {interface} {new_property_values:#?}");
                    }
                    "InterfacesRemoved" => {
                        let (obj, interfaces): (zvariant::OwnedObjectPath, Vec<String>) =
                            body.deserialize()?;
                        println!("InterfacesRemoved {obj} {interfaces:#?}");
                        if interfaces.contains(&"org.freedesktop.UDisks2.Filesystem".into()) {
                            send.send(FsEvent::FilesystemRemoved(obj)).await?;
                        }
                    }
                    "InterfacesAdded" => {
                        let (obj, interfaces_and_properties): (
                            zvariant::OwnedObjectPath,
                            HashMap<String, HashMap<String, zvariant::Value>>,
                        ) = body.deserialize()?;
                        println!(
                            "InterfacesAdded {obj} {:?}",
                            interfaces_and_properties.keys()
                        );
                        if interfaces_and_properties
                            .get("org.freedesktop.UDisks2.Filesystem")
                            .is_some()
                        {
                            send.send(FsEvent::FilesystemAdded(obj)).await?;
                        }
                    }
                    other => {
                        println!("Unknown signal: {other}");
                        println!("Data signature is: {:?}", body.signature());
                    }
                }
            }
        }
    }

    Ok(())
}
