//! Two-threaded iceoryx2 request/response round-trip bench.

use std::hint::black_box;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use iceoryx2::port::client::Client;
use iceoryx2::prelude::*;

use crate::harness::{self, Bench, RunCfg};
use crate::pin;

/// Registry name used on the CLI.
pub const NAME: &str = "ice-rr-2t";

/// Main (client) → worker (server) → main round-trip over one
/// iceoryx2 request/response service — the pattern's structural
/// advantage over pub/sub (`ice-ps-2t`), which needs a service
/// per direction. Both ends spin (`receive` is non-blocking
/// only), so this measures the spin-spin fast path, not
/// park/wake cost.
///
/// See `ice-ps-1t` for the service-name / `/dev/shm` residue notes.
pub struct IceReqRes2Thread {
    client: Client<ipc::Service, u64, (), u64, ()>,
    // Declared after the port: fields drop in declaration order,
    // and dropping the node while ports live trips iceoryx2's
    // dead-node detection (a noisy [W] on the next node creation).
    _node: Node<ipc::Service>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
    counter: u64,
}

impl IceReqRes2Thread {
    /// Spawn the echo-server worker, optionally pinning it to
    /// `worker_cpu`.
    pub fn new(worker_cpu: Option<usize>) -> Self {
        let name = format!("iiac-perf-ice-rr-2t-{}", std::process::id());
        // Explicit default config: skips the global config-file
        // lookup (and its "No config file was loaded" warning) and
        // keeps the bench hermetic — results can't be skewed by a
        // machine-local iceoryx2.toml.
        let node = NodeBuilder::new()
            .config(&iceoryx2::config::Config::default())
            .create::<ipc::Service>()
            .expect("iceoryx2 node");
        let client = node
            .service_builder(&name.as_str().try_into().expect("service name"))
            .request_response::<u64, u64>()
            .open_or_create()
            .expect("iceoryx2 service")
            .client_builder()
            .create()
            .expect("client");

        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker_name = name.clone();
        let worker = thread::spawn(move || {
            pin::pin_current(worker_cpu);
            // The worker opens the same service through its own node,
            // as a second process would.
            let node = NodeBuilder::new()
                .config(&iceoryx2::config::Config::default())
                .create::<ipc::Service>()
                .expect("iceoryx2 worker node");
            let server = node
                .service_builder(&worker_name.as_str().try_into().expect("service name"))
                .request_response::<u64, u64>()
                .open_or_create()
                .expect("iceoryx2 worker service")
                .server_builder()
                .create()
                .expect("server");
            while !worker_stop.load(Ordering::Relaxed) {
                if let Some(request) = server.receive().expect("worker receive") {
                    request.send_copy(*request).expect("worker respond");
                } else {
                    core::hint::spin_loop();
                }
            }
        });

        // Handshake: a request sent before the worker's server
        // connects is dropped, and its PendingResponse would never
        // resolve — a lost first request would leave step()
        // spinning forever. Re-request until an echo arrives; each
        // retry's PendingResponse is dropped and closed with it.
        loop {
            let pending = client.send_copy(0).expect("handshake send");
            thread::sleep(std::time::Duration::from_millis(1));
            if pending.receive().expect("handshake receive").is_some() {
                break;
            }
        }

        Self {
            client,
            _node: node,
            stop,
            worker: Some(worker),
            counter: 0,
        }
    }
}

impl Bench for IceReqRes2Thread {
    fn name(&self) -> &str {
        "ice-rr-2t: iceoryx2 req/res round-trip (2 threads)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        let pending = self.client.send_copy(self.counter).expect("send_copy");
        loop {
            if let Some(response) = pending.receive().expect("receive") {
                return black_box(*response);
            }
            core::hint::spin_loop();
        }
    }
}

impl Drop for IceReqRes2Thread {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = IceReqRes2Thread::new(cfg.core_for(1));
    let (hist, outer, inner, duration_s, suspended_s) = harness::run_adaptive(&mut bench, cfg);
    harness::print_report(
        bench.name(),
        outer,
        inner,
        duration_s,
        &hist,
        cfg,
        suspended_s,
    );
}
