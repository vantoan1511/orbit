use orbit_core::command::CoreCommand;
use orbit_core::dispatcher::CoreDispatcher;
use orbit_core::events::CoreEvent;
use orbit_core::kubernetes::manager::KubeManager;
use orbit_core::sink::MpscEventSink;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

pub struct CoreBridge {
    pub event_rx: mpsc::UnboundedReceiver<CoreEvent>,
    pub command_tx: mpsc::Sender<CoreCommand>,
}

pub fn start_core_bridge() -> CoreBridge {
    let (ui_event_tx, ui_event_rx) = mpsc::unbounded_channel::<CoreEvent>();
    let (command_tx, mut command_rx) = mpsc::channel::<CoreCommand>(64);

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(err) => {
                let _ = ui_event_tx.send(CoreEvent::ErrorOccurred {
                    action: "init_runtime".to_string(),
                    message: format!("Failed to create Tokio runtime: {err}"),
                });
                return;
            }
        };

        rt.block_on(async move {
            let (core_tx, mut core_rx) = mpsc::unbounded_channel::<CoreEvent>();
            let sink = Arc::new(MpscEventSink::new(core_tx));
            let manager = Arc::new(RwLock::new(KubeManager::new().await));
            let dispatcher = CoreDispatcher::new(manager, sink);

            dispatcher.dispatch(CoreCommand::Initialize).await;

            loop {
                tokio::select! {
                    Some(event) = core_rx.recv() => {
                        if let CoreEvent::EngineReady {
                            active_cluster_id: Some(ref id),
                            ..
                        } = event
                        {
                            let cluster_id = id.clone();
                            let _ = ui_event_tx.send(event);
                            dispatcher
                                .dispatch(CoreCommand::SwitchCluster {
                                    context_name: Some(cluster_id),
                                })
                                .await;
                            continue;
                        }
                        if ui_event_tx.send(event).is_err() {
                            break;
                        }
                    }
                    Some(cmd) = command_rx.recv() => {
                        dispatcher.dispatch(cmd).await;
                    }
                    else => break,
                }
            }
        });
    });

    CoreBridge {
        event_rx: ui_event_rx,
        command_tx,
    }
}
