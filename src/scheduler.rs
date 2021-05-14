use crate::connector::cache_data::{CacheData, sync};
use crate::connector::{get_implementation, DataConnector};
use crate::settings::Settings;
use actix_rt::time;
use actix_rt::Arbiter;
use parse_duration::parse;
use log::{info, warn, error};
use std::sync::{Arc, Mutex};
use std::process::exit;
use futures_util::StreamExt;
pub struct Scheduler {
    arbiter: Arbiter,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            arbiter: Arbiter::new(),
        }
    }

    pub fn run<F, R>(&mut self, interval: std::time::Duration, mut task: F)
    where
        F: FnMut() -> R + Send + 'static,
        R: std::future::Future<Output = ()> + Send + 'static,
    {
        let future = time::interval(interval)
            .for_each_concurrent(2, move |_| task());
        self.arbiter.send(future);
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.arbiter.stop();
    }
}


pub fn register_saving_scheduler(
    scheduler: &mut Scheduler,
    db: Arc<CacheData>,
    config: &Settings,
) {

    let frequency = match parse(&*config.database.batching_frequency) {
        Ok(e) => e,
        Err(_) => {
            error!("An error occurred while parsing the duration.");
            exit(1);
        }
    };
    info!(
        "Enabling the saving scheduler, with a delay of {}s",
        frequency.as_secs()
    );
    scheduler.run(frequency, move || {

        let data = db.clone();
        async move {

            sync(data).await.unwrap()
        }
    });
}


