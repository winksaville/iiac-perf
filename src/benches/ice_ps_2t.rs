//! Two-threaded iceoryx2 publish/subscribe round-trip bench.

use std::hint::black_box;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::prelude::*;

use crate::harness::{self, Bench, RunCfg};
use crate::pin;

/// Registry name used on the CLI.
pub const NAME: &str = "ice-ps-2t";

/// Main → worker → main round-trip over two iceoryx2
/// publish/subscribe services (one per direction — pub/sub has no
/// return path, so the echo needs a second service). Both ends
/// spin-receive (`Subscriber::receive` is non-blocking only), so
/// this measures the spin-spin fast path, comparable to a hot
/// `mpsc-2t`, not park/wake cost.
///
/// See `ice-ps-1t` for the service-name / `/dev/shm` residue notes.
pub struct IcePubSub2Thread {
    req_tx: Publisher<ipc::Service, u64, ()>,
    resp_rx: Subscriber<ipc::Service, u64, ()>,
    // Declared after the ports: fields drop in declaration order,
    // and dropping the node while ports live trips iceoryx2's
    // dead-node detection (a noisy [W] on the next node creation).
    _node: Node<ipc::Service>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
    counter: u64,
}

/// Build the two service names (req, resp) shared by both threads.
fn service_names() -> (String, String) {
    let pid = std::process::id();
    (
        format!("iiac-perf-ice-ps-2t-req-{pid}"),
        format!("iiac-perf-ice-ps-2t-resp-{pid}"),
    )
}

/// Open a pub/sub `u64` service by name on `node`.
fn open_service(
    node: &Node<ipc::Service>,
    name: &str,
) -> iceoryx2::service::port_factory::publish_subscribe::PortFactory<ipc::Service, u64, ()> {
    node.service_builder(&name.try_into().expect("service name"))
        .publish_subscribe::<u64>()
        .open_or_create()
        .expect("iceoryx2 service")
}

impl IcePubSub2Thread {
    /// Spawn the echo worker, optionally pinning it to `worker_cpu`.
    pub fn new(worker_cpu: Option<usize>) -> Self {
        let (req_name, resp_name) = service_names();
        // Explicit default config: skips the global config-file
        // lookup (and its "No config file was loaded" warning) and
        // keeps the bench hermetic — results can't be skewed by a
        // machine-local iceoryx2.toml.
        let node = NodeBuilder::new()
            .config(&iceoryx2::config::Config::default())
            .create::<ipc::Service>()
            .expect("iceoryx2 node");
        let req_tx = open_service(&node, &req_name)
            .publisher_builder()
            .create()
            .expect("req publisher");
        let resp_rx = open_service(&node, &resp_name)
            .subscriber_builder()
            .create()
            .expect("resp subscriber");

        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker = thread::spawn(move || {
            pin::pin_current(worker_cpu);
            // The worker opens the same services through its own node,
            // as a second process would.
            let node = NodeBuilder::new()
                .config(&iceoryx2::config::Config::default())
                .create::<ipc::Service>()
                .expect("iceoryx2 worker node");
            let req_rx = open_service(&node, &req_name)
                .subscriber_builder()
                .create()
                .expect("req subscriber");
            let resp_tx = open_service(&node, &resp_name)
                .publisher_builder()
                .create()
                .expect("resp publisher");
            while !worker_stop.load(Ordering::Relaxed) {
                if let Some(sample) = req_rx.receive().expect("worker receive") {
                    resp_tx.send_copy(*sample).expect("worker send_copy");
                } else {
                    core::hint::spin_loop();
                }
            }
        });

        // Handshake: pub/sub has no history here, so a sample
        // published before the worker's subscriber connects is
        // silently dropped — and a lost first request would leave
        // step() spinning forever. Re-ping until an echo arrives,
        // then drain the extra echoes from repeated pings.
        loop {
            req_tx.send_copy(0).expect("handshake send");
            thread::sleep(std::time::Duration::from_millis(1));
            if resp_rx.receive().expect("handshake receive").is_some() {
                break;
            }
        }
        thread::sleep(std::time::Duration::from_millis(10));
        while resp_rx.receive().expect("handshake drain").is_some() {}

        Self {
            req_tx,
            resp_rx,
            _node: node,
            stop,
            worker: Some(worker),
            counter: 0,
        }
    }
}

impl Bench for IcePubSub2Thread {
    fn name(&self) -> &str {
        "ice-ps-2t: iceoryx2 pub/sub round-trip (2 threads)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        self.req_tx.send_copy(self.counter).expect("send_copy");
        loop {
            if let Some(sample) = self.resp_rx.receive().expect("receive") {
                return black_box(*sample);
            }
            core::hint::spin_loop();
        }
    }
}

impl Drop for IcePubSub2Thread {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = IcePubSub2Thread::new(cfg.core_for(1));
    let (hist, outer, inner, duration_s, suspended_s) = harness::run_adaptive(&mut bench, cfg);
    harness::print_report(
        bench.name(),
        outer,
        inner,
        duration_s,
        &hist,
        cfg.overhead,
        suspended_s,
    );
}
