#[cfg(feature = "stats")]
use lazy_static::lazy_static;
#[cfg(feature = "stats")]
use hyper::{Server as HyperServer, Body, Response as HyperResponse, Request as HyperRequest};
#[cfg(feature = "stats")]
use hyper::service::{make_service_fn, service_fn};
#[cfg(feature = "stats")]
use std::convert::Infallible;
#[cfg(feature = "stats")]
use prometheus::{Encoder, IntCounterVec, CounterVec, IntGaugeVec, Opts, Registry, GaugeVec};
#[cfg(feature = "stats")]
use std::time::{Duration};
// Enum for metric types
#[cfg(feature = "stats")]
pub enum WorkerPoolMetricType {
    TotalWorkers,
    Active,
}

#[cfg(feature = "stats")]
pub enum WorkerPoolTaskType {
    Executed,
    Panic,
    Queued,
    Dispatched,
}

#[cfg(feature = "stats")]
lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    static ref WORKER_POOL_METRICS: IntGaugeVec = IntGaugeVec::new(
        Opts::new("worker_pool_metrics", "Worker pool metrics"),
        &["type"]
    )
    .unwrap();
    static ref WORKER_POOL_TASKS: IntGaugeVec = IntGaugeVec::new(
        Opts::new("worker_pool_tasks", "Worker pool tasks metrics"),
        &["type"]
    )
    .unwrap();

    static ref WORKER_UTILIZATION: GaugeVec = GaugeVec::new(
        Opts::new("worker_utilization", "Worker utilization"),
        &["worker_id"]
    ).unwrap();

}

#[cfg(feature = "stats")]
fn register_metrics() {
    for metric in vec![
        Box::new(WORKER_POOL_METRICS.clone()),
        Box::new(WORKER_POOL_TASKS.clone()),
    ] {
        REGISTRY.register(metric).expect("Failed to register metric");
    }

    // Custom registration for WORKER_UTILIZATION
    REGISTRY
        .register(Box::new(WORKER_UTILIZATION.clone()))
        .expect("Failed to register WORKER_UTILIZATION metric");
}

#[cfg(feature = "stats")]
pub async fn start_metrics_server() -> Result<(), Box<dyn std::error::Error>> {
    register_metrics();
    async fn metrics(_req: HyperRequest<Body>) -> Result<HyperResponse<Body>, Infallible> {
        println!("metrics scrape");
        let encoder = prometheus::TextEncoder::new();
        let metric_families = REGISTRY.gather();  // Use custom registry
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer).unwrap();
        let output = String::from_utf8(buffer.clone()).unwrap();

        Ok(HyperResponse::new(Body::from(buffer)))
    }

    let metrics_addr = ([0, 0, 0, 0], 9091).into();
    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(metrics)) });
    let metrics_server = HyperServer::bind(&metrics_addr).serve(make_svc);

    println!("Metrics server started at http://{}", metrics_addr);

    metrics_server.await?;

    Ok(())
}

#[cfg(feature = "stats")]
pub fn increment_task(task_type: WorkerPoolTaskType) {
    let label = match task_type {
        WorkerPoolTaskType::Executed => "executed",
        WorkerPoolTaskType::Panic => "panic",
        WorkerPoolTaskType::Dispatched => "dispatched",
        WorkerPoolTaskType::Queued => "queued",
    };
    WORKER_POOL_TASKS.with_label_values(&[label]).inc();
}

#[cfg(feature = "stats")]
pub fn decrement_task_queue() {
    WORKER_POOL_TASKS.with_label_values(&["queued"]).dec();
}

#[cfg(feature = "stats")]
pub fn set_worker_pool_metric(metric_type: WorkerPoolMetricType, value: usize) {
    let label = match metric_type {
        WorkerPoolMetricType::TotalWorkers => "total_workers",
        WorkerPoolMetricType::Active => "active",
    };
    WORKER_POOL_METRICS.with_label_values(&[label]).set(value as i64);
}

#[cfg(feature = "stats")]
pub fn update_worker_utilization(worker_id: usize, active_time: Duration, total_time: Duration) {
    let utilization = if total_time.as_nanos() > 0 {
        (active_time.as_nanos() as f64 / total_time.as_nanos() as f64) * 100.0
    } else {
        0.0
    };
    WORKER_UTILIZATION.with_label_values(&[&worker_id.to_string()]).set(utilization);
}