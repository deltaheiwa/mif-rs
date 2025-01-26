extern crate lru;

use std::collections::HashMap;
use std::sync::Arc;
use logfather::error;
use tokio::sync::Mutex;
use tokio::runtime;
use lru::LruCache;
use poise::serenity_prelude as serenity;
use prometheus::{Registry, IntGauge, Gauge, TextEncoder, Encoder};
use serenity::prelude::TypeMapKey;
use sysinfo::{Pid, System};


#[derive(Clone)]
pub struct Data {
    pub db_pool: sqlx::SqlitePool,
    pub prefix_cache: Arc<Mutex<LruCache<String, String>>>,
    pub language_cache: Arc<Mutex<LruCache<String, String>>>,
    pub wolvesville_client: Arc<reqwest::Client>,
    pub custom_emojis: HashMap<String, serenity::Emoji>,
}

impl TypeMapKey for Data {
    type Value = Arc<Self>;
}

pub struct CustomColor;

impl CustomColor {
    pub const CYAN: serenity::Color = serenity::Color::from_rgb(0, 255, 255);
}

pub struct CustomEmoji;

impl CustomEmoji {
    pub const SINGLE_ROSE: &'static str = "single_rose";
    pub const LETS_PLAY: &'static str = "lets_play";
    pub const LOADING: &'static str = "loading";
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type PartialContext<'a> = poise::PartialContext<'a, Data, Error>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub type ApiResult<T> = Result<Option<T>, reqwest::Error>;

pub struct SystemMetrics {
    registry: Registry,
    cpu_usage: Gauge,
    memory_usage: Gauge,
    thread_count: IntGauge,
}

impl SystemMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();
        let cpu_usage = Gauge::new("cpu_usage", "Overall CPU utilization percentage").unwrap();
        let memory_usage = Gauge::new("memory_usage", "Total system memory usage in MB").unwrap();
        let thread_count = IntGauge::new("thread_count", "Total amount of threads").unwrap();

        registry.register(Box::new(cpu_usage.clone())).unwrap();
        registry.register(Box::new(memory_usage.clone())).unwrap();
        registry.register(Box::new(thread_count.clone())).unwrap();

        Self {
            registry,
            cpu_usage,
            memory_usage,
            thread_count,
        }
    }

    pub fn update(&self, system: &mut System, pid: Pid) {
        system.refresh_all();

        if let Some(process) = system.process(pid) {
            self.memory_usage.set((process.memory() as f32 / 2.0_f32.powf(20.0)) as f64);
            self.cpu_usage.set(process.cpu_usage() as f64);
            self.thread_count.set(runtime::Handle::current().metrics().num_workers() as i64);
        } else {
            error!("Could not find process with pid {}", pid);
        }
    }


    pub fn render_metrics(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();

        encoder.encode(&metric_families, &mut buffer)
            .expect("Failed to encode metrics");

        String::from_utf8(buffer).expect("Failed to convert metrics to string")
    }
}

pub struct MetricsManager {
    pub system_metrics: SystemMetrics,
}

impl MetricsManager {
    pub fn new() -> Self {
        let system_metrics = SystemMetrics::new();
        Self { system_metrics }
    }
}
