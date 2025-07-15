use std::collections::HashMap;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use chrono::{DateTime, Duration, Utc};
use logfather::{error, info, warn};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use crate::db::jobs::{get_all_jobs, delete_job, add_job};

pub type JobFn = Arc<
    dyn Fn(serde_json::Value) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>
    + Send
    + Sync,
>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Schedule {
    Once(DateTime<Utc>),
    Interval(Duration),
    Cron(String),
}

impl Schedule {
    pub fn next_run(&self, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            Schedule::Once(t) => (*t > after).then_some(*t),
            Schedule::Interval(duration) => Some(after + *duration),
            Schedule::Cron(s) => {
                cron::Schedule::from_str(s).ok()?.after(&after).next()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDefinition {
    pub id: Uuid,
    pub name: String,
    pub schedule: Schedule,
    pub created_at: DateTime<Utc>,
    pub args: serde_json::Value,
}

pub struct ScheduledJob {
    definition: JobDefinition,
    last_run: Option<DateTime<Utc>>,
    next_run: DateTime<Utc>,
}

#[derive(Clone, Default)]
pub struct JobRegistry {
    jobs: HashMap<String, JobFn>,
}

struct SchedulerState {
    jobs: HashMap<Uuid, ScheduledJob>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a job function that accepts arguments via `serde_json::Value`.
    pub fn register<F, Fut>(&mut self, name: &str, f: F)
    where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let job_fn: JobFn = Arc::new(move |v| Box::pin(f(v)));
        self.jobs.insert(name.to_string(), job_fn);
    }

    /// Registers a job function that takes no arguments.
    pub fn register_no_args<F, Fut>(&mut self, name: &str, f: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let job_fn: JobFn = Arc::new(move |_args: serde_json::Value| Box::pin(f()));
        self.jobs.insert(name.to_string(), job_fn);
    }

    pub fn get(&self, name: &str) -> Option<JobFn> {
        self.jobs.get(name).cloned()
    }
}

#[derive(Clone)]
pub struct Scheduler {
    pool: Arc<SqlitePool>,
    state: Arc<Mutex<SchedulerState>>,
    registry: Arc<JobRegistry>,
}
impl Scheduler {
    pub fn new(pool: Arc<SqlitePool>, registry: Arc<JobRegistry>) -> Self {
        Self {
            pool,
            state: Arc::new(Mutex::new(SchedulerState {
                jobs: HashMap::new(),
            })),
            registry,
        }
    }

    /// Load all jobs from the database into memory. Preferably called at startup.
    pub async fn load_from_store(&self, ) -> anyhow::Result<()> {
        info!("Loading jobs from store...");
        let jobs = get_all_jobs(&*self.pool) .await?;
        let mut state = self.state.lock().await;
        let now = Utc::now();

        for def in jobs {
            if let Some(next_run) = def.schedule.next_run(now) {
                info!("Loading job: id = {}, name = {}, next_run = {}",
                    &def.id, &def.name, &next_run);
                state.jobs.insert(
                    def.id,
                    ScheduledJob {
                        definition: def,
                        last_run: None,
                        next_run,
                    },
                );
            } else {
                warn!("Skipping job: id = {}, name = {} (no valid next run time)",
                    def.id, def.name);
            }
        }
        info!("Finished loading {} jobs", state.jobs.len());
        Ok(())
    }

    /// The main execution loop.
    ///
    /// # Arguments
    /// * `token` - A cancellation token to gracefully shut down the scheduler.
    pub async fn run(&self, token: CancellationToken) {
        info!("Scheduler run loop started");
        let check_interval = std::time::Duration::from_secs(60);

        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    info!("Scheduler received cancellation signal. Shutting down.");
                    break;
                }
                _ = tokio::time::sleep(check_interval) => {
                    self.tick().await;
                }
            }
        }
    }

    /// A single check for due jobs.
    async fn tick(&self) {
        let now = Utc::now();
        let mut state = self.state.lock().await;
        let mut jobs_to_remove = Vec::new();

        for job in state.jobs.values_mut() {
            if job.next_run <= now {
                self.execute_job(job);

                if let Some(next_run) = job.definition.schedule.next_run(now) {
                    job.next_run = next_run;
                } else {
                    jobs_to_remove.push(job.definition.id);
                }
            }
        }

        if !jobs_to_remove.is_empty() {
            for id in jobs_to_remove {
                info!("Removing job from schedule: id = {}", id);
                state.jobs.remove(&id);
                if let Err(e) = delete_job(&*self.pool, id).await {
                    error!("Failed to remove job from db: {}", e);
                }
            }
        }
    }

    fn execute_job(&self, job: &ScheduledJob) {
        info!("Executing job: id = {}, name = {}",
            job.definition.id, job.definition.name);
        let args = job.definition.args.clone();
        match self.registry.get(&job.definition.name) {
            Some(job_fn) => {
                tokio::spawn(async move {
                    if let Err(e) = job_fn(args).await {
                        error!("Failed to execute job: {}", e);
                    }
                });
            }
            None => {
                warn!("Job name {} not found in registry. Skipping execution.",
                    job.definition.name);
            }
        }
    }

    /// Adds a new job to the scheduler.
    ///
    /// # Arguments
    /// * `name` - The name of the job to add.
    /// * `schedule` - The schedule for the job, which can be a one-time run, an interval, or a cron expression.
    pub async fn add_job(
        &self,
        name: String,
        schedule: Schedule,
        args: &serde_json::Value,
    ) -> anyhow::Result<JobDefinition> {
        if self.registry.get(&name).is_none() {
            return Err(anyhow::anyhow!("Job '{}' not found in registry", name));
        }

        let now = Utc::now();
        let def = JobDefinition {
            id: Uuid::new_v4(),
            name,
            schedule,
            created_at: now,
            args: args.clone(),
        };

        add_job(&*self.pool, &def).await?;

        if let Some(next_run) = def.schedule.next_run(now) {
            let mut state = self.state.lock().await;
            state.jobs.insert(def.id, ScheduledJob {
                definition: def.clone(),
                last_run: None,
                next_run,
            });
            info!("Added job to schedule: id = {}, name = {}, next_run = {}",
                def.id, def.name, next_run);
        }

        Ok(def)
    }

    /// Removes a job from the scheduler and the database.
    ///
    /// # Arguments
    /// * `id` - The UUID of the job to remove.
    pub async fn remove_job(&self, id: Uuid) -> anyhow::Result<()> {
        delete_job(&*self.pool, id)
            .await?;

        let mut state = self.state.lock().await;
        if state.jobs.remove(&id).is_some() {
            info!("Removed job from schedule: id = {}", id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Job not found in active schedule"))
        }
    }
}

