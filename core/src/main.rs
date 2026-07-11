mod ipc;
mod kubernetes;

use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;
use ipc::bridge::{AuthInfo, Bridge};
use ipc::events::OrbitEvent;
use kubernetes::manager::KubeManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Orbit Core Engine starting up...");

    // Retrieve authentication details from stdin
    let mut auth = AuthInfo::from_stdin();
    // Allow overriding authentication details from command-line arguments
    auth.override_from_cli();

    println!("Connecting to port: {}", auth.nl_port);

    let mut bridge = Bridge::connect(&auth).await?;
    println!("Orbit Core Engine connected to Neutralinojs WebSocket server.");

    // Initialize KubeManager
    let kube_manager = Arc::new(RwLock::new(KubeManager::new().await));

    // Broadcast that the core is connected and ready
    Bridge::send_event(
        &bridge.writer,
        &bridge.token,
        &OrbitEvent::EngineConnected {
            status: "ready".to_string(),
            message: "Orbit Engine is connected and ready.".to_string(),
        },
    ).await?;

    // Also send initial clusters list and active context
    {
        let manager = kube_manager.read().await;
        let clusters = manager.get_clusters();
        let active_cluster_id = manager.active_context.clone();

        let _ = Bridge::send_event(
            &bridge.writer,
            &bridge.token,
            &OrbitEvent::ClustersUpdated { clusters },
        ).await;

        let _ = Bridge::send_event(
            &bridge.writer,
            &bridge.token,
            &OrbitEvent::ActiveClusterChanged { active_cluster_id },
        ).await;
    }

    // Message processing loop
    loop {
        let msg = match Bridge::read_message(&mut bridge.reader, &bridge.writer).await {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("WebSocket error occurred or connection closed: {:?}", e);
                break;
            }
        };

        if msg.event.as_deref() == Some("windowClose") {
            log::info!("Received windowClose, shutting down.");
            break;
        }

        // Re-broadcast connection status when a client connects to ensure the frontend receives it
        if msg.event.as_deref() == Some("appClientConnect") || msg.event.as_deref() == Some("clientConnect") {
            let writer = bridge.writer.clone();
            let token = bridge.token.clone();
            let manager = kube_manager.clone();
            tokio::spawn(async move {
                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::EngineConnected {
                        status: "ready".to_string(),
                        message: "Orbit Engine is connected and ready.".to_string(),
                    },
                ).await;

                let r_manager = manager.read().await;
                let clusters = r_manager.get_clusters();
                let active_cluster_id = r_manager.active_context.clone();

                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::ClustersUpdated { clusters },
                ).await;

                let _ = Bridge::send_event(
                    &writer,
                    &token,
                    &OrbitEvent::ActiveClusterChanged { active_cluster_id },
                ).await;
            });
        }

        // Handle extension events from frontend
        if let Some(event_name) = msg.event.as_deref() {
            let writer = bridge.writer.clone();
            let token = bridge.token.clone();
            let manager = kube_manager.clone();
            
            match event_name {
                "getClusters" => {
                    tokio::spawn(async move {
                        let r_manager = manager.read().await;
                        let clusters = r_manager.get_clusters();
                        let active_cluster_id = r_manager.active_context.clone();

                        let _ = Bridge::send_event(
                            &writer,
                            &token,
                            &OrbitEvent::ClustersUpdated { clusters },
                        ).await;

                        let _ = Bridge::send_event(
                            &writer,
                            &token,
                            &OrbitEvent::ActiveClusterChanged { active_cluster_id },
                        ).await;
                    });
                }
                "switchCluster" => {
                    let ext_data = msg.data.clone();
                    tokio::spawn(async move {
                        let cluster_id = ext_data
                            .and_then(|d| d.get("clusterId").cloned())
                            .and_then(|v| v.as_str().map(|s| s.to_string()));

                        if let Some(id) = cluster_id {
                            let mut w_manager = manager.write().await;
                            match w_manager.switch_context(&id).await {
                                Ok(()) => {
                                    let active_cluster_id = w_manager.active_context.clone();
                                    let clusters = w_manager.get_clusters();
                                    let client = w_manager.active_client.clone();
                                    drop(w_manager);

                                    let _ = Bridge::send_event(
                                        &writer,
                                        &token,
                                        &OrbitEvent::ActiveClusterChanged { active_cluster_id },
                                    ).await;

                                    let _ = Bridge::send_event(
                                        &writer,
                                        &token,
                                        &OrbitEvent::ClustersUpdated { clusters },
                                    ).await;

                                    // Refresh namespaces and pods for new active client
                                    if let Some(ref client) = client {
                                        if let Ok(namespaces) = kubernetes::list_namespaces(client).await {
                                            let _ = Bridge::send_event(
                                                &writer,
                                                &token,
                                                &OrbitEvent::NamespacesUpdated { namespaces },
                                            ).await;
                                        }
                                        if let Ok(pods) = kubernetes::list_pods(client, None).await {
                                            let _ = Bridge::send_event(
                                                &writer,
                                                &token,
                                                &OrbitEvent::PodsUpdated { pods },
                                            ).await;
                                        }
                                        if let Ok(nodes) = kubernetes::list_nodes(client).await {
                                            let _ = Bridge::send_event(
                                                &writer,
                                                &token,
                                                &OrbitEvent::NodesUpdated { nodes },
                                            ).await;
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error switching cluster: {:?}", e);
                                    let _ = Bridge::send_event(
                                        &writer,
                                        &token,
                                        &OrbitEvent::ErrorOccurred {
                                            message: format!("Failed to switch cluster: {}", e),
                                        },
                                    ).await;
                                }
                            }
                        }
                    });
                }
                "addCluster" => {
                    let ext_data = msg.data.clone();
                    tokio::spawn(async move {
                        let file_path = ext_data
                            .and_then(|d| d.get("filePath").cloned())
                            .and_then(|v| v.as_str().map(|s| s.to_string()));

                        if let Some(path) = file_path {
                            let mut w_manager = manager.write().await;
                            match w_manager.add_kubeconfig_file(&path).await {
                                Ok(()) => {
                                    let clusters = w_manager.get_clusters();
                                    let active_cluster_id = w_manager.active_context.clone();
                                    let client = w_manager.active_client.clone();
                                    drop(w_manager);

                                    let _ = Bridge::send_event(
                                        &writer,
                                        &token,
                                        &OrbitEvent::ClustersUpdated { clusters },
                                    ).await;

                                    let _ = Bridge::send_event(
                                        &writer,
                                        &token,
                                        &OrbitEvent::ActiveClusterChanged { active_cluster_id },
                                    ).await;

                                    // Refresh namespaces and pods for new active client
                                    if let Some(ref client) = client {
                                        if let Ok(namespaces) = kubernetes::list_namespaces(client).await {
                                            let _ = Bridge::send_event(
                                                &writer,
                                                &token,
                                                &OrbitEvent::NamespacesUpdated { namespaces },
                                            ).await;
                                        }
                                        if let Ok(pods) = kubernetes::list_pods(client, None).await {
                                            let _ = Bridge::send_event(
                                                &writer,
                                                &token,
                                                &OrbitEvent::PodsUpdated { pods },
                                            ).await;
                                        }
                                        if let Ok(nodes) = kubernetes::list_nodes(client).await {
                                            let _ = Bridge::send_event(
                                                &writer,
                                                &token,
                                                &OrbitEvent::NodesUpdated { nodes },
                                            ).await;
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error adding cluster: {:?}", e);
                                    let _ = Bridge::send_event(
                                        &writer,
                                        &token,
                                        &OrbitEvent::ErrorOccurred {
                                            message: format!("Failed to add cluster: {}", e),
                                        },
                                    ).await;
                                }
                            }
                        }
                    });
                }
                "getNamespaces" => {
                    tokio::spawn(async move {
                        let client = {
                            let r_manager = manager.read().await;
                            r_manager.active_client.clone()
                        };
                        if let Some(ref client) = client {
                            match kubernetes::list_namespaces(client).await {
                                Ok(namespaces) => {
                                    let _ = Bridge::send_event(
                                        &writer,
                                        &token,
                                        &OrbitEvent::NamespacesUpdated { namespaces },
                                    ).await;
                                }
                                Err(e) => {
                                    eprintln!("Error listing namespaces: {:?}", e);
                                }
                            }
                        }
                    });
                }
                "getPods" => {
                    let ext_data = msg.data.clone();
                    tokio::spawn(async move {
                        let namespace = ext_data
                            .and_then(|d| d.get("namespace").cloned())
                            .and_then(|v| v.as_str().map(|s| s.to_string()));

                        let client = {
                            let r_manager = manager.read().await;
                            r_manager.active_client.clone()
                        };
                        if let Some(ref client) = client {
                            match kubernetes::list_pods(client, namespace).await {
                                Ok(pods) => {
                                    let _ = Bridge::send_event(
                                        &writer,
                                        &token,
                                        &OrbitEvent::PodsUpdated { pods },
                                    ).await;
                                }
                                Err(e) => {
                                    eprintln!("Error listing pods: {:?}", e);
                                }
                            }
                        }
                    });
                }
                "getDeployments" => {
                    let ext_data = msg.data.clone();
                    tokio::spawn(async move {
                        let namespace = ext_data
                            .and_then(|d| d.get("namespace").cloned())
                            .and_then(|v| v.as_str().map(|s| s.to_string()));

                        let client = {
                            let r_manager = manager.read().await;
                            r_manager.active_client.clone()
                        };
                        if let Some(ref client) = client {
                            match kubernetes::list_deployments(client, namespace).await {
                                Ok(deployments) => {
                                    let _ = Bridge::send_event(
                                        &writer,
                                        &token,
                                        &OrbitEvent::DeploymentsUpdated { deployments },
                                    ).await;
                                }
                                Err(e) => {
                                    eprintln!("Error listing deployments: {:?}", e);
                                }
                            }
                        }
                    });
                }
                "getStatefulSets" => {
                    let ext_data = msg.data.clone();
                    tokio::spawn(async move {
                        let namespace = ext_data
                            .and_then(|d| d.get("namespace").cloned())
                            .and_then(|v| v.as_str().map(|s| s.to_string()));

                        let client = {
                            let r_manager = manager.read().await;
                            r_manager.active_client.clone()
                        };
                        if let Some(ref client) = client {
                            match kubernetes::list_statefulsets(client, namespace).await {
                                Ok(stateful_sets) => {
                                    let _ = Bridge::send_event(
                                        &writer,
                                        &token,
                                        &OrbitEvent::StatefulSetsUpdated { stateful_sets },
                                    ).await;
                                }
                                Err(e) => {
                                    eprintln!("Error listing statefulsets: {:?}", e);
                                }
                            }
                        }
                    });
                }
                "getDaemonSets" => {
                    let ext_data = msg.data.clone();
                    tokio::spawn(async move {
                        let namespace = ext_data
                            .and_then(|d| d.get("namespace").cloned())
                            .and_then(|v| v.as_str().map(|s| s.to_string()));

                        let client = {
                            let r_manager = manager.read().await;
                            r_manager.active_client.clone()
                        };
                        if let Some(ref client) = client {
                            match kubernetes::list_daemonsets(client, namespace).await {
                                Ok(daemon_sets) => {
                                    let _ = Bridge::send_event(
                                        &writer,
                                        &token,
                                        &OrbitEvent::DaemonSetsUpdated { daemon_sets },
                                    ).await;
                                }
                                Err(e) => {
                                    eprintln!("Error listing daemonsets: {:?}", e);
                                }
                            }
                        }
                    });
                }
                "getReplicaSets" => {
                    let ext_data = msg.data.clone();
                    tokio::spawn(async move {
                        let namespace = ext_data
                            .and_then(|d| d.get("namespace").cloned())
                            .and_then(|v| v.as_str().map(|s| s.to_string()));

                        let client = {
                            let r_manager = manager.read().await;
                            r_manager.active_client.clone()
                        };
                        if let Some(ref client) = client {
                            match kubernetes::list_replicasets(client, namespace).await {
                                Ok(replica_sets) => {
                                    let _ = Bridge::send_event(
                                        &writer,
                                        &token,
                                        &OrbitEvent::ReplicaSetsUpdated { replica_sets },
                                    ).await;
                                }
                                Err(e) => {
                                    eprintln!("Error listing replicasets: {:?}", e);
                                }
                            }
                        }
                    });
                }
                "getJobs" => {
                    let ext_data = msg.data.clone();
                    tokio::spawn(async move {
                        let namespace = ext_data
                            .and_then(|d| d.get("namespace").cloned())
                            .and_then(|v| v.as_str().map(|s| s.to_string()));

                        let client = {
                            let r_manager = manager.read().await;
                            r_manager.active_client.clone()
                        };
                        if let Some(ref client) = client {
                            match kubernetes::list_jobs(client, namespace).await {
                                Ok(jobs) => {
                                    let _ = Bridge::send_event(
                                        &writer,
                                        &token,
                                        &OrbitEvent::JobsUpdated { jobs },
                                    ).await;
                                }
                                Err(e) => {
                                    eprintln!("Error listing jobs: {:?}", e);
                                }
                            }
                        }
                    });
                }
                "getCronJobs" => {
                    let ext_data = msg.data.clone();
                    tokio::spawn(async move {
                        let namespace = ext_data
                            .and_then(|d| d.get("namespace").cloned())
                            .and_then(|v| v.as_str().map(|s| s.to_string()));

                        let client = {
                            let r_manager = manager.read().await;
                            r_manager.active_client.clone()
                        };
                        if let Some(ref client) = client {
                            match kubernetes::list_cronjobs(client, namespace).await {
                                Ok(cron_jobs) => {
                                    let _ = Bridge::send_event(
                                        &writer,
                                        &token,
                                        &OrbitEvent::CronJobsUpdated { cron_jobs },
                                    ).await;
                                }
                                Err(e) => {
                                    eprintln!("Error listing cronjobs: {:?}", e);
                                }
                            }
                        }
                    });
                }
                "getServices" => {
                    let ext_data = msg.data.clone();
                    tokio::spawn(async move {
                        let namespace = ext_data
                            .and_then(|d| d.get("namespace").cloned())
                            .and_then(|v| v.as_str().map(|s| s.to_string()));

                        let client = {
                            let r_manager = manager.read().await;
                            r_manager.active_client.clone()
                        };
                        if let Some(ref client) = client {
                            match kubernetes::list_services(client, namespace).await {
                                Ok(services) => {
                                    let _ = Bridge::send_event(
                                        &writer,
                                        &token,
                                        &OrbitEvent::ServicesUpdated { services },
                                    ).await;
                                }
                                Err(e) => {
                                    eprintln!("Error listing services: {:?}", e);
                                }
                            }
                        }
                    });
                }
                "getNodes" => {
                    tokio::spawn(async move {
                        let client = {
                            let r_manager = manager.read().await;
                            r_manager.active_client.clone()
                        };
                        if let Some(ref client) = client {
                            match kubernetes::list_nodes(client).await {
                                Ok(nodes) => {
                                    let _ = Bridge::send_event(
                                        &writer,
                                        &token,
                                        &OrbitEvent::NodesUpdated { nodes },
                                    ).await;
                                }
                                Err(e) => {
                                    eprintln!("Error listing nodes: {:?}", e);
                                }
                            }
                        }
                    });
                }
                _ => {}
            }
        }

        tokio::spawn(async move {
            println!("Handled received message: {:?}", msg);
        });
    }

    Ok(())
}
