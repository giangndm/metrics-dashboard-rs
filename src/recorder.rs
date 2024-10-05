use metrics::{Key, Metadata, Recorder};
use parking_lot::RwLock;
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};

use crate::DashboardOptions;

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
    pub key: String,
    typ: MetricType,
    pub desc: Option<String>,
    pub unit: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct MetricValue {
    pub key: String,
    #[serde(rename = "value", skip_serializing_if = "Option::is_none")]
    pub value_u64: Option<u64>,
    #[serde(rename = "value", skip_serializing_if = "Option::is_none")]
    pub value_f64: Option<f64>,
}

#[derive(Default)]
struct DashboardStorage {
    counters: HashMap<String, SimpleCounter>,
    gauges: HashMap<String, SimpleGauge>,
    histograms: HashMap<String, SimpleHistogram>,
    metrics: HashMap<String, MetricMeta>,
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
    pub options: DashboardOptions,
    storage: Arc<RwLock<DashboardStorage>>,
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
        Self {
            options: opts,
            storage: Default::default(),
        }
    }

    /// Retrieves the metrics as a vector of `MetricMeta`.
    ///
    /// # Returns
    ///
    /// A vector of `MetricMeta`.
    pub fn metrics(&self) -> Vec<MetricMeta> {
        let mut res = vec![];
        let storage = self.storage.read();
        for (_key, meta) in storage.metrics.iter() {
            res.push(meta.clone());
        }
        res.sort_by_cached_key(|m: &MetricMeta| m.key.clone());
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
        let mut storage = self.storage.write();
        let mut data = vec![];
        for key in keys {
            if let Some(meta) = storage.metrics.get(key) {
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
        let mut storage = self.storage.write();
        if let Some(metric) = storage.metrics.get_mut(key.as_str()) {
            metric.desc = Some(description.to_string());
        } else {
            storage.metrics.insert(
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
        let mut storage = self.storage.write();
        if let Some(metric) = storage.metrics.get_mut(key.as_str()) {
            metric.desc = Some(description.to_string())
        } else {
            storage.metrics.insert(
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
        let mut storage = self.storage.write();
        if let Some(metric) = storage.metrics.get_mut(key.as_str()) {
            metric.desc = Some(description.to_string())
        } else {
            storage.metrics.insert(
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

    fn register_counter(&self, key: &Key, _metadata: &Metadata<'_>) -> metrics::Counter {
        let mut storage = self.storage.write();
        if !storage.metrics.contains_key(key.name()) {
            storage.metrics.insert(
                key.name().to_string(),
                MetricMeta {
                    key: key.name().to_string(),
                    typ: MetricType::Counter,
                    desc: None,
                    unit: None,
                },
            );
        }

        metrics::Counter::from_arc(storage.get_counter(key.name()).into())
    }

    fn register_gauge(&self, key: &Key, _metadata: &Metadata<'_>) -> metrics::Gauge {
        let mut storage = self.storage.write();
        if !storage.metrics.contains_key(key.name()) {
            storage.metrics.insert(
                key.name().to_string(),
                MetricMeta {
                    key: key.name().to_string(),
                    typ: MetricType::Gauge,
                    desc: None,
                    unit: None,
                },
            );
        }

        metrics::Gauge::from_arc(storage.get_gauge(key.name()).into())
    }

    fn register_histogram(&self, key: &Key, _metadata: &Metadata<'_>) -> metrics::Histogram {
        let mut storage = self.storage.write();
        if !storage.metrics.contains_key(key.name()) {
            storage.metrics.insert(
                key.name().to_string(),
                MetricMeta {
                    key: key.name().to_string(),
                    typ: MetricType::Histogram,
                    desc: None,
                    unit: None,
                },
            );
        }

        metrics::Histogram::from_arc(storage.get_histogram(key.name()).into())
    }
}

#[cfg(test)]
mod tests {
    use metrics::Level;

    use super::*;
    use std::{sync::Arc, thread};

    fn default_meta() -> Metadata<'static> {
        Metadata::new("test-target", Level::DEBUG, None)
    }

    #[test]
    fn test_counter_basic_operations() {
        let recorder = DashboardRecorder::new(DashboardOptions::default());
        let counter =
            recorder.register_counter(&Key::from_static_name("test_counter"), &default_meta());

        counter.increment(1);
        let values = recorder.metrics_value(vec!["test_counter"]);

        assert_eq!(values.len(), 1);
        assert_eq!(values[0].key, "test_counter");
        assert_eq!(values[0].value_u64, Some(1));
        assert_eq!(values[0].value_f64, None);
    }

    #[test]
    fn test_gauge_basic_operations() {
        let recorder = DashboardRecorder::new(DashboardOptions::default());
        let gauge = recorder.register_gauge(&Key::from_static_name("test_gauge"), &default_meta());

        gauge.set(42.5);
        let values = recorder.metrics_value(vec!["test_gauge"]);

        assert_eq!(values.len(), 1);
        assert_eq!(values[0].key, "test_gauge");
        assert_eq!(values[0].value_u64, None);
        assert_eq!(values[0].value_f64, Some(42.5));
    }

    #[test]
    fn test_metadata_descriptions() {
        let recorder = DashboardRecorder::new(DashboardOptions::default());
        let key_name = metrics::KeyName::from_const_str("test_counter");
        let description = metrics::SharedString::const_str("A test counter");

        recorder.describe_counter(key_name.clone(), None, description.clone());

        let metrics = recorder.metrics();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].key, "test_counter");
        assert_eq!(metrics[0].desc, Some("A test counter".to_string()));
    }

    #[test]
    fn test_nonexistent_metric() {
        let recorder = DashboardRecorder::new(DashboardOptions::default());
        let values = recorder.metrics_value(vec!["nonexistent"]);
        assert!(values.is_empty());
    }

    #[test]
    fn test_multiple_metrics() {
        let recorder = DashboardRecorder::new(DashboardOptions::default());

        let counter =
            recorder.register_counter(&Key::from_static_name("test_counter"), &default_meta());
        let gauge = recorder.register_gauge(&Key::from_static_name("test_gauge"), &default_meta());

        counter.increment(5);
        gauge.set(3.14);

        let values = recorder.metrics_value(vec!["test_counter", "test_gauge"]);
        assert_eq!(values.len(), 2);

        let counter_value = values.iter().find(|v| v.key == "test_counter").unwrap();
        let gauge_value = values.iter().find(|v| v.key == "test_gauge").unwrap();

        assert_eq!(counter_value.value_u64, Some(5));
        assert_eq!(gauge_value.value_f64, Some(3.14));
    }

    #[test]
    fn test_concurrent_access() {
        let recorder = Arc::new(DashboardRecorder::new(DashboardOptions::default()));

        let num_threads = 10;
        let iterations = 1000;
        let mut handles = vec![];

        for _ in 0..num_threads {
            let recorder_clone = Arc::clone(&recorder);

            handles.push(thread::spawn(move || {
                let gauge = recorder_clone
                    .register_gauge(&Key::from_static_name("concurrent_gauge"), &default_meta());
                let counter = recorder_clone.register_counter(
                    &Key::from_static_name("concurrent_counter"),
                    &default_meta(),
                );
                for i in 0..iterations {
                    counter.increment(1);
                    gauge.set(i as f64);

                    // Also read values concurrently
                    let _values = recorder_clone
                        .metrics_value(vec!["concurrent_counter", "concurrent_gauge"]);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let final_values = recorder.metrics_value(vec!["concurrent_counter"]);
        assert_eq!(
            final_values[0].value_u64,
            Some((num_threads * iterations) as u64)
        );
    }

    #[test]
    fn test_metrics_sorted_order() {
        let recorder = DashboardRecorder::new(DashboardOptions::default());

        // Register metrics in non-alphabetical order
        recorder.register_counter(&Key::from_static_name("z_counter"), &default_meta());
        recorder.register_counter(&Key::from_static_name("a_counter"), &default_meta());
        recorder.register_counter(&Key::from_static_name("m_counter"), &default_meta());

        let metrics = recorder.metrics();

        // Verify metrics are returned in alphabetical order
        assert_eq!(metrics[0].key, "a_counter");
        assert_eq!(metrics[1].key, "m_counter");
        assert_eq!(metrics[2].key, "z_counter");
    }

    #[test]
    fn test_metric_type_consistency() {
        let recorder = DashboardRecorder::new(DashboardOptions::default());

        // Register same key name with different types
        recorder.register_counter(&Key::from_static_name("test_metric"), &default_meta());
        recorder.register_gauge(&Key::from_static_name("test_metric"), &default_meta());

        let metrics = recorder.metrics();
        assert_eq!(metrics.len(), 1); // Should only have one metric

        match metrics[0].typ {
            MetricType::Counter => (),
            _ => panic!("Metric type changed unexpectedly"),
        }
    }
}

// Optionally, you might want to add some benchmark tests
// #[cfg(test)]
// mod benchmarks {
//     use super::*;
//     use test::Bencher;

//     #[bench]
//     fn bench_counter_increment(b: &mut Bencher) {
//         let recorder = DashboardRecorder::new(DashboardOptions::default());
//         let counter =
//             recorder.register_counter(&Key::from_static_name("bench_counter"), &default_meta());

//         b.iter(|| {
//             counter.increment(1);
//         });
//     }

//     #[bench]
//     fn bench_metrics_value_retrieval(b: &mut Bencher) {
//         let recorder = DashboardRecorder::new(DashboardOptions::default());
//         let counter =
//             recorder.register_counter(&Key::from_static_name("bench_counter"), &default_meta());
//         counter.increment(1);

//         b.iter(|| {
//             recorder.metrics_value(vec!["bench_counter"]);
//         });
//     }
// }
