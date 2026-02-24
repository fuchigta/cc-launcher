use crate::config::AppConfig;
use crate::headless;
use crate::models::{ExecutionSource, ScheduleConfig, ScheduleExpression};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct SchedulerManager {
    scheduler: Arc<RwLock<JobScheduler>>,
    app_handle: tauri::AppHandle,
}

impl SchedulerManager {
    pub async fn new(app_handle: tauri::AppHandle) -> Result<Self, String> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| format!("Failed to create scheduler: {}", e))?;
        scheduler
            .start()
            .await
            .map_err(|e| format!("Failed to start scheduler: {}", e))?;
        Ok(Self {
            scheduler: Arc::new(RwLock::new(scheduler)),
            app_handle,
        })
    }

    pub async fn reload_all(&self, schedules: &[ScheduleConfig]) -> Result<(), String> {
        // Shut down existing scheduler and create a new one
        {
            let mut sched = self.scheduler.write().await;
            let _ = sched.shutdown().await;
        }

        let new_scheduler = JobScheduler::new()
            .await
            .map_err(|e| format!("Failed to create scheduler: {}", e))?;
        new_scheduler
            .start()
            .await
            .map_err(|e| format!("Failed to start scheduler: {}", e))?;

        {
            let mut sched = self.scheduler.write().await;
            *sched = new_scheduler;
        }

        for schedule in schedules {
            if schedule.enabled {
                self.add_schedule(schedule).await?;
            }
        }
        Ok(())
    }

    async fn add_schedule(&self, config: &ScheduleConfig) -> Result<(), String> {
        let app_handle = self.app_handle.clone();
        let prompt = config.prompt.clone();
        let working_dir = config.working_dir.clone();
        let claude_args = config.claude_args.clone();
        let schedule_id = config.id.clone();
        let schedule_name = config.name.clone();

        macro_rules! make_callback {
            () => {{
                let app_handle = app_handle.clone();
                let prompt = prompt.clone();
                let working_dir = working_dir.clone();
                let claude_args = claude_args.clone();
                let schedule_id = schedule_id.clone();
                let schedule_name = schedule_name.clone();
                move |_uuid, _lock| {
                    let app = app_handle.clone();
                    let p = prompt.clone();
                    let wd = working_dir.clone();
                    let args = claude_args.clone();
                    let sid = schedule_id.clone();
                    let sname = schedule_name.clone();
                    Box::pin(async move {
                        let timeout_secs = AppConfig::load().timeout_secs;
                        let _ = headless::execute(
                            &p,
                            wd.as_deref(),
                            &args,
                            ExecutionSource::Schedule {
                                id: sid,
                                name: sname,
                            },
                            &app,
                            timeout_secs,
                        )
                        .await;
                    })
                }
            }};
        }

        let cron_expr_owned;
        let job = match &config.expression {
            ScheduleExpression::Cron { expression } => {
                Job::new_async(expression.as_str(), make_callback!())
                    .map_err(|e| format!("Invalid cron expression: {}", e))?
            }
            ScheduleExpression::Interval { seconds } => {
                let duration = std::time::Duration::from_secs(*seconds);
                Job::new_repeated_async(duration, make_callback!())
                    .map_err(|e| format!("Invalid interval: {}", e))?
            }
            ScheduleExpression::DailyAt { time } => {
                let parts: Vec<&str> = time.split(':').collect();
                if parts.len() != 2 {
                    return Err(format!("Invalid time format: {}", time));
                }
                let hour: u32 = parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid hour: {}", parts[0]))?;
                let minute: u32 = parts[1]
                    .parse()
                    .map_err(|_| format!("Invalid minute: {}", parts[1]))?;
                cron_expr_owned = format!("0 {} {} * * *", minute, hour);
                Job::new_async(cron_expr_owned.as_str(), make_callback!())
                    .map_err(|e| format!("Invalid daily schedule: {}", e))?
            }
        };

        let sched = self.scheduler.write().await;
        sched
            .add(job)
            .await
            .map_err(|e| format!("Failed to add job: {}", e))?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn shutdown(&self) -> Result<(), String> {
        let mut sched = self.scheduler.write().await;
        sched
            .shutdown()
            .await
            .map_err(|e| format!("Failed to shutdown scheduler: {}", e))
    }
}
