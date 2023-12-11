use metrics::{Key, Recorder};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use self::{counter::SimpleCounter, gauge::SimpleGauge, histogram::SimpleHistogram};

mod counter;
mod gauge;
mod histogram;

#[derive(Debug, Serialize, Clone)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

#[derive(Debug, Serialize, Clone)]
pub struct MetricMeta {
    key: String,
    typ: MetricType,
    desc: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct MetricValue {
    key: String,
    #[serde(rename = "value", skip_serializing_if = "Option::is_none")]
    value_u64: Option<u64>,
    #[serde(rename = "value", skip_serializing_if = "Option::is_none")]
    value_f64: Option<f64>,
}

#[derive(Default)]
struct DashboardStorage {
    counters: HashMap<String, SimpleCounter>,
    gauges: HashMap<String, SimpleGauge>,
    histograms: HashMap<String, SimpleHistogram>,
}

impl DashboardStorage {
    fn get_counter(&mut self, key: &str) -> SimpleCounter {
        let entry = self
            .counters
            .entry(key.to_string())
            .or_default();
        entry.clone()
    }

    fn get_gause(&mut self, key: &str) -> SimpleGauge {
        let entry = self
            .gauges
            .entry(key.to_string())
            .or_default();
        entry.clone()
    }

    fn get_histogram(&mut self, key: &str) -> SimpleHistogram {
        let entry = self
            .histograms
            .entry(key.to_string())
            .or_default();
        entry.clone()
    }
}

#[derive(Clone)]
pub struct DashboardRecorder {
    storage: Arc<RwLock<DashboardStorage>>,
    metrics: Arc<RwLock<HashMap<String, MetricMeta>>>,
}

impl DashboardRecorder {
    pub fn new() -> Self {
        Self {
            storage: Default::default(),
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn metrics(&self) -> Vec<MetricMeta> {
        let mut res = vec![];
        let metrics = &*self.metrics.read().expect("Should lock");
        for (_key, meta) in metrics.iter() {
            res.push(meta.clone());
        }
        res.sort_by_cached_key(|m: &MetricMeta| m.key.clone());
        res
    }

    pub fn metrics_value(&self, keys: Vec<&str>) -> Vec<MetricValue> {
        let mut storage = self.storage.write().expect("Should lock");
        let metrics = self.metrics.read().expect("Should lock");
        let mut data = vec![];
        for key in keys {
            if let Some(meta) = metrics.get(key) {
                match meta.typ {
                    MetricType::Counter => {
                        let counter = storage.get_counter(key);
                        data.push(MetricValue {
                            key: key.to_string(),
                            value_u64: Some(counter.value()),
                            value_f64: None,
                        });
                    }
                    MetricType::Gauge => {
                        let gauge = storage.get_gause(key);
                        data.push(MetricValue {
                            key: key.to_string(),
                            value_u64: None,
                            value_f64: Some((gauge.value() * 100.0).round() / 100.0),
                        });
                    }
                    MetricType::Histogram => {
                        // let _histogram = self
                        //     .registry
                        //     .get_or_create_histogram(&metrics::Key::from(key.to_string()), |a| {
                        //         a.clone()
                        //     });
                        // TODO
                    }
                };
            }
        }
        data
    }
}

impl Recorder for DashboardRecorder {
    fn describe_counter(
        &self,
        key: metrics::KeyName,
        _unit: Option<metrics::Unit>,
        description: metrics::SharedString,
    ) {
        let mut metrics = self.metrics.write().expect("Should ok");
        if let Some(metric) = metrics.get_mut(key.as_str()) {
            metric.desc = Some(description.to_string());
        } else {
            metrics.insert(
                key.as_str().to_string(),
                MetricMeta {
                    key: key.as_str().to_string(),
                    typ: MetricType::Counter,
                    desc: Some(description.to_string()),
                },
            );
        }
    }

    fn describe_gauge(
        &self,
        key: metrics::KeyName,
        _unit: Option<metrics::Unit>,
        description: metrics::SharedString,
    ) {
        let mut metrics = self.metrics.write().expect("Should ok");
        if let Some(metric) = metrics.get_mut(key.as_str()) {
            metric.desc = Some(description.to_string())
        } else {
            metrics.insert(
                key.as_str().to_string(),
                MetricMeta {
                    key: key.as_str().to_string(),
                    typ: MetricType::Gauge,
                    desc: Some(description.to_string()),
                },
            );
        }
    }

    fn describe_histogram(
        &self,
        key: metrics::KeyName,
        _unit: Option<metrics::Unit>,
        description: metrics::SharedString,
    ) {
        let mut metrics = self.metrics.write().expect("Should ok");
        if let Some(metric) = metrics.get_mut(key.as_str()) {
            metric.desc = Some(description.to_string())
        } else {
            metrics.insert(
                key.as_str().to_string(),
                MetricMeta {
                    key: key.as_str().to_string(),
                    typ: MetricType::Histogram,
                    desc: Some(description.to_string()),
                },
            );
        }
    }

    fn register_counter(&self, key: &Key) -> metrics::Counter {
        let mut metrics = self.metrics.write().expect("Should ok");
        if !metrics.contains_key(key.name()) {
            metrics.insert(
                key.name().to_string(),
                MetricMeta {
                    key: key.name().to_string(),
                    typ: MetricType::Counter,
                    desc: None,
                },
            );
        }

        metrics::Counter::from_arc(
            self.storage
                .write()
                .expect("Should lock")
                .get_counter(key.name())
                .into(),
        )
    }

    fn register_gauge(&self, key: &Key) -> metrics::Gauge {
        let mut metrics = self.metrics.write().expect("Should ok");
        if !metrics.contains_key(key.name()) {
            metrics.insert(
                key.name().to_string(),
                MetricMeta {
                    key: key.name().to_string(),
                    typ: MetricType::Gauge,
                    desc: None,
                },
            );
        }

        metrics::Gauge::from_arc(
            self.storage
                .write()
                .expect("Should lock")
                .get_gause(key.name())
                .into(),
        )
    }

    fn register_histogram(&self, key: &Key) -> metrics::Histogram {
        let mut metrics = self.metrics.write().expect("Should ok");
        if !metrics.contains_key(key.name()) {
            metrics.insert(
                key.name().to_string(),
                MetricMeta {
                    key: key.name().to_string(),
                    typ: MetricType::Histogram,
                    desc: None,
                },
            );
        }

        metrics::Histogram::from_arc(
            self.storage
                .write()
                .expect("Should lock")
                .get_histogram(key.name())
                .into(),
        )
    }
}
