use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use axum::Router;
use tokio::net::TcpListener;

use crate::Result;

#[derive(Clone, Debug)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
}

#[derive(Clone, Default)]
pub struct ShutdownToken {
    state: Arc<ShutdownState>,
}

struct ShutdownState {
    requested: AtomicBool,
    wait_lock: Mutex<()>,
    changed: Condvar,
}

impl Default for ShutdownState {
    fn default() -> Self {
        Self {
            requested: AtomicBool::new(false),
            wait_lock: Mutex::new(()),
            changed: Condvar::new(),
        }
    }
}

impl ShutdownToken {
    pub fn request(&self) {
        if !self.state.requested.swap(true, Ordering::SeqCst) {
            self.state.changed.notify_all();
        }
    }

    pub fn is_requested(&self) -> bool {
        self.state.requested.load(Ordering::SeqCst)
    }

    pub fn wait_timeout(&self, timeout: Duration) -> bool {
        if self.is_requested() {
            return true;
        }
        let lock = self.state.wait_lock.lock().expect("shutdown lock poisoned");
        let _ = self
            .state
            .changed
            .wait_timeout_while(lock, timeout, |_| !self.is_requested())
            .expect("shutdown lock poisoned");
        self.is_requested()
    }
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let terminate = async {
            match signal(SignalKind::terminate()) {
                Ok(mut signal) => {
                    signal.recv().await;
                }
                Err(_) => std::future::pending::<()>().await,
            }
        };
        tokio::select! {
            _ = ctrl_c => {}
            _ = terminate => {}
        }
    }

    #[cfg(not(unix))]
    ctrl_c.await;
}

pub async fn serve(addr: &str, router: Router) -> Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|err| format!("failed to bind {addr}: {err}"))?;
    axum::serve(listener, router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|err| format!("server error: {err}").into())
}

/// Stop product workers and close streams when a shutdown signal arrives,
/// before Axum waits for its active requests to drain.
pub async fn serve_with_shutdown<F>(addr: &str, router: Router, cleanup: F) -> Result<()>
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|err| format!("failed to bind {addr}: {err}"))?;
    axum::serve(listener, router.into_make_service())
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            cleanup.await;
        })
        .await
        .map_err(|err| format!("server error: {err}").into())
}
