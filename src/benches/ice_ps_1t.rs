//! Single-threaded iceoryx2 publish/subscribe round-trip bench.

use std::hint::black_box;

use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::prelude::*;

use crate::harness::{self, Bench, RunCfg};

/// Registry name used on the CLI.
pub const NAME: &str = "ice-ps-1t";

/// Same-thread publish-then-receive through one iceoryx2
/// publish/subscribe service. Measures pure transport overhead
/// (shared-memory sample loan, copy-in, queue, receive) with no
/// scheduler interaction.
///
/// iceoryx2 service names are machine-global and runs leave a
/// persistent management segment in `/dev/shm` plus dirs under
/// `/tmp/iceoryx2`; the pid in the service name keeps concurrent
/// runs from colliding, and clean exits tear the service down.
pub struct IcePubSub1Thread {
    publisher: Publisher<ipc::Service, u64, ()>,
    subscriber: Subscriber<ipc::Service, u64, ()>,
    // Declared after the ports: fields drop in declaration order,
    // and dropping the node while ports live trips iceoryx2's
    // dead-node detection (a noisy [W] on the next node creation).
    _node: Node<ipc::Service>,
    counter: u64,
}

impl IcePubSub1Thread {
    /// Create the node, service, and both ports.
    pub fn new() -> Self {
        // Explicit default config: skips the global config-file
        // lookup (and its "No config file was loaded" warning) and
        // keeps the bench hermetic — results can't be skewed by a
        // machine-local iceoryx2.toml.
        let node = NodeBuilder::new()
            .config(&iceoryx2::config::Config::default())
            .create::<ipc::Service>()
            .expect("iceoryx2 node");
        let name = format!("iiac-perf-ice-ps-1t-{}", std::process::id());
        let service = node
            .service_builder(&name.as_str().try_into().expect("service name"))
            .publish_subscribe::<u64>()
            .open_or_create()
            .expect("iceoryx2 service");
        let publisher = service.publisher_builder().create().expect("publisher");
        let subscriber = service.subscriber_builder().create().expect("subscriber");
        Self {
            _node: node,
            publisher,
            subscriber,
            counter: 0,
        }
    }
}

impl Bench for IcePubSub1Thread {
    fn name(&self) -> &str {
        "iceoryx2 pub/sub round-trip (1 thread)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        self.publisher.send_copy(self.counter).expect("send_copy");
        loop {
            if let Some(sample) = self.subscriber.receive().expect("receive") {
                return black_box(*sample);
            }
            core::hint::spin_loop();
        }
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = IcePubSub1Thread::new();
    let (hist, outer, inner, duration_s) = harness::run_adaptive(&mut bench, cfg);
    harness::print_report(bench.name(), outer, inner, duration_s, &hist, cfg.overhead);
}
