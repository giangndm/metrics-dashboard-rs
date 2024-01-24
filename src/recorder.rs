use metrics::{Key, Recorder};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{ChartType, DashboardOptions};

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
    unit: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct MetricValue {
    key: String,
    #[serde(rename = "value", skip_serializing_if = "Option::is_none")]
    value_u64: Option<u64>,
    #[serde(rename = "value", skip_serializing_if = "Option::is_none")]
    value_f64: Option<f64>,
    // unit: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ChartMeta {
    desc: Option<String>,
    key: String,
    chart_type: ChartType,
    unit: Option<String>,
}

#[derive(Default)]
struct DashboardStorage {
    counters: HashMap<String, SimpleCounter>,
    gauges: HashMap<String, SimpleGauge>,
    histograms: HashMap<String, SimpleHistogram>,
}

impl DashboardStorage {
    fn get_counter(&mut self, key: &str) -> SimpleCounter {
        let entry = self.counters.entry(key.to_string()).or_default();
        entry.clone()
    }

    fn get_gauge(&mut self, key: &str) -> SimpleGauge {
        let entry = self.gauges.entry(key.to_string()).or_default();
        entry.clone()
    }

    fn get_histogram(&mut self, key: &str) -> SimpleHistogram {
        let entry = self.histograms.entry(key.to_string()).or_default();
        entry.clone()
    }
}

#[derive(Clone)]
pub struct DashboardRecorder {
    storage: Arc<RwLock<DashboardStorage>>,
    metrics: Arc<RwLock<HashMap<String, MetricMeta>>>,
    charts: Arc<HashMap<String, ChartMeta>>,
    options: DashboardOptions,
}

/// The `DashboardRecorder` struct represents a recorder for metrics dashboard.
/// It provides methods for adding bound keys, retrieving metrics, and retrieving metric values.
impl DashboardRecorder {
    /// Creates a new instance of `DashboardRecorder`.
    ///
    /// # Returns
    ///
    /// A new instance of `DashboardRecorder`.
    pub fn new(opts: DashboardOptions) -> Self {
        let mut charts = HashMap::<String, ChartMeta>::new();
        for chart in opts.charts.iter() {
            let metric = match chart {
                ChartType::Line { metric, .. } => metric,
                ChartType::Bar { metric, .. } => metric,
            };
            charts.insert(
                metric.clone(),
                ChartMeta {
                    desc: None,
                    key: metric.clone(),
                    chart_type: chart.clone(),
                    unit: None,
                },
            );
        }

        Self {
            storage: Default::default(),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            options: opts,
            charts: Arc::new(charts),
        }
    }

    /// Retrieves the metrics as a vector of `MetricMeta`.
    ///
    /// # Returns
    ///
    /// A vector of `MetricMeta`.
    pub fn metrics(&self) -> Vec<MetricMeta> {
        let mut res = vec![];
        let metrics = &*self.metrics.read().expect("Should lock");
        for (_key, meta) in metrics.iter() {
            res.push(meta.clone());
        }
        res.sort_by_cached_key(|m: &MetricMeta| m.key.clone());
        res
    }

    pub fn charts(&self) -> Vec<ChartMeta> {
        let mut res = vec![];
        let charts = &*self.charts;
        let metrics = &*self.metrics.read().expect("Should lock");
        for (_key, meta) in charts.iter() {
            if let Some(metric) = metrics.get(&meta.key) {
                let mut meta = meta.clone();
                meta.unit = metric.unit.clone();
                meta.desc = metric.desc.clone();
                res.push(meta.clone());
            }
        }

        if self.options.include_default {
            let mut max_keys = HashMap::<String, bool>::new();
            for (_k, chart) in charts.iter() {
                match &chart.chart_type {
                    ChartType::Line { max_metric, .. } => {
                        if let Some(max_metric) = max_metric {
                            max_keys.insert(max_metric.clone(), true);
                        }
                    }
                    ChartType::Bar { max_metric, .. } => {
                        if let Some(max_metric) = max_metric {
                            max_keys.insert(max_metric.clone(), true);
                        }
                    }
                }
            }

            for (_k, metric) in metrics.iter() {
                if !charts.contains_key(&metric.key) && !max_keys.contains_key(&metric.key) {
                    res.push(ChartMeta {
                        desc: metric.desc.clone(),
                        key: metric.key.clone(),
                        chart_type: ChartType::Line {
                            metric: metric.key.clone(),
                            max_metric: None,
                        },
                        unit: metric.unit.clone(),
                    });
                }
            }
        }
        res.sort_by_cached_key(|m: &ChartMeta| m.key.clone());
        res
    }

    /// Retrieves the metric values for the specified keys.
    ///
    /// # Arguments
    ///
    /// * `keys` - The keys to retrieve metric values for.
    ///
    /// # Returns
    ///
    /// A vector of `MetricValue`.
    pub fn metrics_value(&self, keys: Vec<&str>) -> Vec<MetricValue> {
        let mut storage = self.storage.write().expect("Should lock");
        let metrics = self.metrics.read().expect("Should lock");
        let mut data = vec![];
        for key in keys {
            if let Some(meta) = metrics.get(key) {
                println!("key: {:?}, meta: {:?}", key, meta);
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
                        let gauge = storage.get_gauge(key);
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
        unit: Option<metrics::Unit>,
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
                    unit: unit.map(|u| u.as_canonical_label().to_string()),
                },
            );
        }
    }

    fn describe_gauge(
        &self,
        key: metrics::KeyName,
        unit: Option<metrics::Unit>,
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
                    unit: unit.map(|u| u.as_canonical_label().to_string()),
                },
            );
        }
    }

    fn describe_histogram(
        &self,
        key: metrics::KeyName,
        unit: Option<metrics::Unit>,
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
                    unit: unit.map(|u| u.as_canonical_label().to_string()),
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
                    unit: None,
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
                    unit: None,
                },
            );
        }

        metrics::Gauge::from_arc(
            self.storage
                .write()
                .expect("Should lock")
                .get_gauge(key.name())
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
                    unit: None,
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
