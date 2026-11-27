use crate::headless;
use crate::models::{ExecutionSource, ScheduleConfig, ScheduleExpression};
use chrono::Local;
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
                        let _ = headless::execute(
                            &p,
                            wd.as_deref(),
                            &args,
                            ExecutionSource::Schedule {
                                id: sid,
                                name: sname,
                            },
                            &app,
                        )
                        .await;
                    })
                }
            }};
        }

        let cron_expr_owned;
        let job = match &config.expression {
            ScheduleExpression::Cron { expression } => {
                cron_expr_owned = normalize_cron_expression(expression.as_str())?;
                Job::new_async_tz(cron_expr_owned.as_str(), Local, make_callback!())
                    .map_err(|e| format!("Invalid cron expression: {}", e))?
            }
            ScheduleExpression::Interval { seconds } => {
                let duration = std::time::Duration::from_secs(*seconds);
                Job::new_repeated_async(duration, make_callback!())
                    .map_err(|e| format!("Invalid interval: {}", e))?
            }
            ScheduleExpression::DailyAt { time } => {
                let (hour, minute) = parse_daily_time(time)?;
                cron_expr_owned = format!("0 {} {} * * *", minute, hour);
                Job::new_async_tz(cron_expr_owned.as_str(), Local, make_callback!())
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

fn parse_daily_time(time: &str) -> Result<(u32, u32), String> {
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
    if hour > 23 {
        return Err(format!("Invalid hour: {} (must be 0-23)", hour));
    }
    if minute > 59 {
        return Err(format!("Invalid minute: {} (must be 0-59)", minute));
    }
    Ok((hour, minute))
}

fn normalize_cron_expression(expr: &str) -> Result<String, String> {
    let field_count = expr.split_whitespace().count();
    match field_count {
        5 => Ok(format!("0 {}", expr)),
        6 => Ok(expr.to_string()),
        _ => Err(format!(
            "Invalid cron expression: expected 5 or 6 fields, got {}. \
             Use standard cron format (e.g. '0 9 * * 1-5' for weekdays at 9am)",
            field_count
        )),
    }
}

// PROBE（ゲートの検証用の一時変更、revert する）
#[allow(dead_code)]
fn retry_delay_secs(attempt: u32) -> u32 {
    if attempt >= 3 {
        60
    } else {
        attempt * 10
    }
}

// PROBE（ゲートの合算の検証用の一時変更、revert する）
#[allow(dead_code)]
fn window_slots(start: u32, end: u32, step: u32) -> u32 {
    if step == 0 || end <= start {
        return 0;
    }
    let count = (end - start) / step + 1;
    if count > 100 {
        100
    } else {
        count * step
    }
}

// PROBE（ゲートの合算の検証用の一時変更、revert する）
#[allow(dead_code)]
fn probe_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // PROBE（ゲートの検証用の一時変更、revert する）
    #[test]
    fn retry_delay_secs_grows_then_caps() {
        assert_eq!(retry_delay_secs(0), 0);
        assert_eq!(retry_delay_secs(1), 10);
        assert_eq!(retry_delay_secs(2), 20);
        assert_eq!(retry_delay_secs(3), 60);
        assert_eq!(retry_delay_secs(4), 60);
    }

    // PROBE（ゲートの合算の検証用の一時変更、revert する）
    #[test]
    fn window_slots_basic() {
        assert_eq!(window_slots(0, 10, 0), 0);
        assert_eq!(window_slots(10, 5, 2), 0);
        assert_eq!(window_slots(2, 10, 4), 12);
    }

    #[test]
    fn normalize_5field_cron_adds_seconds() {
        assert_eq!(
            normalize_cron_expression("0 9 * * 1-5").unwrap(),
            "0 0 9 * * 1-5"
        );
        assert_eq!(
            normalize_cron_expression("0 9 * * *").unwrap(),
            "0 0 9 * * *"
        );
        assert_eq!(
            normalize_cron_expression("30 8 * * MON-FRI").unwrap(),
            "0 30 8 * * MON-FRI"
        );
    }

    #[test]
    fn normalize_6field_cron_is_unchanged() {
        assert_eq!(
            normalize_cron_expression("0 0 9 * * 1-5").unwrap(),
            "0 0 9 * * 1-5"
        );
        assert_eq!(
            normalize_cron_expression("0 0 9 * * *").unwrap(),
            "0 0 9 * * *"
        );
    }

    #[test]
    fn normalize_invalid_field_count_returns_error() {
        assert!(normalize_cron_expression("9 * *").is_err());
        assert!(normalize_cron_expression("0 0 9 * * * *").is_err());
    }

    #[test]
    fn parse_daily_time_valid() {
        assert_eq!(parse_daily_time("9:00").unwrap(), (9, 0));
        assert_eq!(parse_daily_time("23:59").unwrap(), (23, 59));
        assert_eq!(parse_daily_time("0:0").unwrap(), (0, 0));
        assert_eq!(parse_daily_time("12:30").unwrap(), (12, 30));
    }

    #[test]
    fn parse_daily_time_cron_expr() {
        let (hour, minute) = parse_daily_time("9:30").unwrap();
        assert_eq!(format!("0 {} {} * * *", minute, hour), "0 30 9 * * *");
    }

    #[test]
    fn parse_daily_time_invalid_format() {
        assert!(parse_daily_time("9").is_err());
        assert!(parse_daily_time("9:00:00").is_err());
        assert!(parse_daily_time("").is_err());
    }

    #[test]
    fn parse_daily_time_invalid_hour() {
        assert!(parse_daily_time("24:00").is_err());
        assert!(parse_daily_time("99:00").is_err());
    }

    #[test]
    fn parse_daily_time_invalid_minute() {
        assert!(parse_daily_time("9:60").is_err());
        assert!(parse_daily_time("0:99").is_err());
    }

    #[test]
    fn parse_daily_time_non_numeric() {
        assert!(parse_daily_time("abc:00").is_err());
        assert!(parse_daily_time("9:xyz").is_err());
    }

    #[test]
    fn normalize_weekday_cron_variations() {
        assert_eq!(
            normalize_cron_expression("0 9 * * MON-FRI").unwrap(),
            "0 0 9 * * MON-FRI"
        );
        assert_eq!(
            normalize_cron_expression("0 9 1,15 * *").unwrap(),
            "0 0 9 1,15 * *"
        );
    }

    mod props {
        use super::super::*;
        use proptest::collection::vec;
        use proptest::prelude::*;

        /// A single non-empty cron-like field (no whitespace inside).
        fn cron_field() -> impl Strategy<Value = String> {
            "[0-9A-Z*/,?-]{1,6}"
        }

        /// `n` cron fields joined by a run of spaces/tabs.
        fn cron_expr(n: usize) -> impl Strategy<Value = String> {
            (vec(cron_field(), n), "[ \t]{1,3}").prop_map(|(fields, sep)| fields.join(&sep))
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(64))]

            #[test]
            fn parse_daily_time_roundtrip(h in 0u32..=23, m in 0u32..=59) {
                let plain = format!("{h}:{m}");
                let padded = format!("{h:02}:{m:02}");
                prop_assert_eq!(parse_daily_time(&plain), Ok((h, m)));
                prop_assert_eq!(parse_daily_time(&padded), Ok((h, m)));
            }

            #[test]
            fn parse_daily_time_rejects_hour_out_of_range(h in 24u32.., m in 0u32..=59) {
                let input = format!("{h}:{m}");
                prop_assert!(parse_daily_time(&input).is_err());
            }

            #[test]
            fn parse_daily_time_rejects_minute_out_of_range(h in 0u32..=23, m in 60u32..) {
                let input = format!("{h}:{m}");
                prop_assert!(parse_daily_time(&input).is_err());
            }

            #[test]
            fn parse_daily_time_rejects_wrong_part_count(
                parts in vec(0u32..=23, 0..6).prop_filter("not exactly 2 parts", |p| p.len() != 2)
            ) {
                let joined: Vec<String> = parts.iter().map(u32::to_string).collect();
                prop_assert!(parse_daily_time(&joined.join(":")).is_err());
            }

            #[test]
            fn normalize_5_fields_prepends_seconds(expr in cron_expr(5)) {
                prop_assert_eq!(normalize_cron_expression(&expr), Ok(format!("0 {expr}")));
            }

            #[test]
            fn normalize_6_fields_is_unchanged(expr in cron_expr(6)) {
                prop_assert_eq!(normalize_cron_expression(&expr), Ok(expr));
            }

            #[test]
            fn normalize_is_idempotent(expr in (5usize..=6).prop_flat_map(cron_expr)) {
                let once = normalize_cron_expression(&expr).unwrap();
                prop_assert_eq!(normalize_cron_expression(&once), Ok(once.clone()));
            }

            #[test]
            fn normalize_rejects_other_field_counts(
                expr in (0usize..12)
                    .prop_filter("not 5 or 6 fields", |n| *n != 5 && *n != 6)
                    .prop_flat_map(cron_expr)
            ) {
                prop_assert!(normalize_cron_expression(&expr).is_err());
            }
        }
    }
}
