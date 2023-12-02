use metrics::{Key, Recorder};
use metrics_util::registry::{
    AtomicStorage, GenerationalAtomicStorage, GenerationalStorage, Registry,
};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc, RwLock},
};

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
    value: u64,
}

#[derive(Clone)]
pub struct DashboardRecorder {
    registry: Arc<Registry<Key, GenerationalAtomicStorage>>,
    metrics: Arc<RwLock<HashMap<String, MetricMeta>>>,
}

impl DashboardRecorder {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Registry::new(GenerationalStorage::new(AtomicStorage))),
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn metrics(&self) -> Vec<MetricMeta> {
        let mut res = vec![];
        let metrics = &*self.metrics.read().expect("Should lock");
        for (_key, meta) in metrics.iter() {
            res.push(meta.clone());
        }
        res
    }

    pub fn metrics_value(&self, keys: Vec<&str>) -> Vec<MetricValue> {
        let metrics = self.metrics.read().expect("Should lock");
        let mut data = vec![];
        for key in keys {
            if let Some(meta) = metrics.get(key) {
                let value = match meta.typ {
                    MetricType::Counter => {
                        let counter = self
                            .registry
                            .get_or_create_counter(&metrics::Key::from(key.to_string()), |a| {
                                a.clone()
                            });
                        counter.get_inner().load(Ordering::Relaxed)
                    }
                    MetricType::Gauge => {
                        let gauge = self
                            .registry
                            .get_or_create_gauge(&metrics::Key::from(key.to_string()), |a| {
                                a.clone()
                            });
                        gauge.get_inner().load(Ordering::Relaxed)
                    }
                    MetricType::Histogram => {
                        panic!("Dont support yet")
                    }
                };

                data.push(MetricValue {
                    key: key.to_string(),
                    value,
                });
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

        self.registry
            .get_or_create_counter(key, |c| c.clone().into())
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

        self.registry.get_or_create_gauge(key, |c| c.clone().into())
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

        self.registry
            .get_or_create_histogram(key, |c| c.clone().into())
    }
}
