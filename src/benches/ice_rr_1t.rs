//! Single-threaded iceoryx2 request/response round-trip bench.

use std::hint::black_box;

use iceoryx2::port::client::Client;
use iceoryx2::port::server::Server;
use iceoryx2::prelude::*;

use crate::harness::{self, Bench, RunCfg};

/// Registry name used on the CLI.
pub const NAME: &str = "ice-rr-1t";

/// Same-thread request → serve → response through one iceoryx2
/// request/response service. Unlike pub/sub (`ice-ps-1t`), one
/// service carries both directions, and each request allocates a
/// `PendingResponse` handle that routes the reply back to its
/// request — this bench prices that extra machinery.
///
/// See `ice-ps-1t` for the service-name / `/dev/shm` residue notes.
pub struct IceReqRes1Thread {
    client: Client<ipc::Service, u64, (), u64, ()>,
    server: Server<ipc::Service, u64, (), u64, ()>,
    // Declared after the ports: fields drop in declaration order,
    // and dropping the node while ports live trips iceoryx2's
    // dead-node detection (a noisy [W] on the next node creation).
    _node: Node<ipc::Service>,
    counter: u64,
}

impl IceReqRes1Thread {
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
        let name = format!("iiac-perf-ice-rr-1t-{}", std::process::id());
        let service = node
            .service_builder(&name.as_str().try_into().expect("service name"))
            .request_response::<u64, u64>()
            .open_or_create()
            .expect("iceoryx2 service");
        let client = service.client_builder().create().expect("client");
        let server = service.server_builder().create().expect("server");
        Self {
            client,
            server,
            _node: node,
            counter: 0,
        }
    }
}

impl Bench for IceReqRes1Thread {
    fn name(&self) -> &str {
        "ice-rr-1t: iceoryx2 req/res round-trip (1 thread)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        let pending = self.client.send_copy(self.counter).expect("send_copy");
        loop {
            if let Some(request) = self.server.receive().expect("server receive") {
                request.send_copy(*request).expect("respond");
                break;
            }
            core::hint::spin_loop();
        }
        loop {
            if let Some(response) = pending.receive().expect("client receive") {
                return black_box(*response);
            }
            core::hint::spin_loop();
        }
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = IceReqRes1Thread::new();
    let (hist, outer, inner, duration_s) = harness::run_adaptive(&mut bench, cfg);
    harness::print_report(bench.name(), outer, inner, duration_s, &hist, cfg.overhead);
}
