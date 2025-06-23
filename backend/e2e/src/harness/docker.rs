// e2e/src/harness/docker.rs
use anyhow::Result;
use testcontainers::{
    clients::Cli,
    Container,
};
use testcontainers_modules::{postgres::Postgres, rabbitmq::RabbitMq};

/// Holds the infrastructure for the lifetime of a test.
pub struct Infra {
    /// &'static so every container can borrow it safely.
    _docker:      &'static Cli,
    _pg:          Container<'static, Postgres>,
    _mq:          Container<'static, RabbitMq>,
    pub db_url:   String,
    pub amqp_url: String,
}

impl Infra {
    /// Spin-up Postgres + RabbitMQ; keep them alive until `Infra` is dropped.
    pub fn spin_up() -> Result<Self> {
        // 1️⃣  One Docker client for everything – leaked to `'static`
        let docker: &'static Cli = Box::leak(Box::new(Cli::default()));

        // 2️⃣  Start the containers (they borrow `docker`)
        let pg = docker.run(Postgres::default());
        let mq = docker.run(RabbitMq::default());

        // 3️⃣  Connection strings
        let db_url = format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            pg.get_host_port_ipv4(5432)
        );
        let amqp_url = format!(
            "amqp://guest:guest@localhost:{}/%2f",
            mq.get_host_port_ipv4(5672)
        );

        Ok(Self { _docker: docker, _pg: pg, _mq: mq, db_url, amqp_url })
    }
}

