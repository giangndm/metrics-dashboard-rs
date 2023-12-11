use metrics::HistogramFn;

#[derive(Debug, Clone, Default)]
pub struct SimpleHistogram {}

impl HistogramFn for SimpleHistogram {
    fn record(&self, _value: f64) {
        //TODO
    }
}
