use sqlx::{Row, SqlitePool};
use uuid::Uuid;
use crate::utils::scheduler::JobDefinition;

pub async fn add_job(pool: &SqlitePool, job: &JobDefinition) -> anyhow::Result<()> {
    let q = r#"
        INSERT INTO jobs (id, name, schedule, created_at, args)
        VALUES ($1, $2, $3, $4, $5);
    "#;
    
    sqlx::query(q)
        .bind(job.id.to_string())
        .bind(&job.name)
        .bind(serde_json::to_string(&job.schedule)?)
        .bind(job.created_at)
        .bind(serde_json::to_string(&job.args)?)
        .execute(pool)
        .await?;
    
    Ok(())
}

pub async fn get_all_jobs(pool: &SqlitePool) -> anyhow::Result<Vec<JobDefinition>> {
    let q = r#"
        SELECT * FROM jobs;
    "#;
    
    let rows = sqlx::query(q).fetch_all(pool).await?;
    
    let jobs: Vec<JobDefinition> = rows.into_iter().map(|row| {
        JobDefinition {
            id: Uuid::parse_str(row.get("id")).unwrap(),
            name: row.get("name"),
            schedule: serde_json::from_str(row.get("schedule")).unwrap(),
            created_at: row.get("created_at"),
            args: serde_json::from_str(row.get("args")).unwrap(),
        }
    }).collect();
    
    Ok(jobs)
}

pub async fn delete_job(pool: &SqlitePool, job_id: Uuid) -> anyhow::Result<()> {
    let q = r#"
        DELETE FROM jobs WHERE id = $1;
    "#;

    sqlx::query(q).bind(job_id.to_string()).execute(pool).await?;
    
    Ok(())
}