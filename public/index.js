import {
  html,
  render,
  useState,
  useEffect,
  useRef,
} from "https://esm.sh/htm/preact/standalone";

const BusChannel = {};
const CachedChannel = {};
const Metrics = {};

const LineChart = ({ idx, metrics, desc, unit }) => {
  const elm = useRef(null);
  const [value, setValue] = useState();
  useEffect(() => {
    if (!elm) {
      return;
    }
    if (!metrics) {
      return;
    }
    const isMulti = metrics?.length > 1;
    const data = {};
    const opts = Object.assign({}, window.ApexOptionsLine);

    metrics?.map((m) => {
      const key = m;
      const value = CachedChannel[key] ? [CachedChannel[key]] : [];
      data[key] = value;
    });

    opts.series = metrics?.map((m) => {
      return {
        name: m,
        data: data[m],
      };
    });

    const chart = new ApexCharts(elm.current, opts);
    chart.render();

    BusChannel[idx] = (date) => {
      metrics?.map((m) => {
        const value = CachedChannel[m] ? CachedChannel[m][1] : 0;
        if (!isMulti) {
          setValue(value);
        }
        data[m].push([date, value]);
        if (data[m].length > 100) {
          data[m].shift();
        }
      });

      const options = {
        series: metrics?.map((m) => {
          return {
            name: m,
            data: data[m],
          };
        }),
      };
      chart.updateOptions(options, false, false);
    };

    return () => {
      delete BusChannel[idx];
    };
  }, [elm, metrics]);

  return html` <div class="col-md-4">
    <div class="box columnbox mt-4">
      <div class="header">
        <h3 class="title">${desc || metrics?.join(",")}</h3>
        ${metrics?.length === 1 &&
        html`<h2 class="subtitle">${value || "--"} ${unit ? unit : ""}</h2>`}
      </div>
      <div ref=${elm}></div>
    </div>
  </div>`;
};

const BarChart = ({ idx, metrics, desc, unit }) => {
  const elm = useRef(null);
  const [value, setValue] = useState();
  useEffect(() => {
    if (!elm) {
      return;
    }
    if (!metrics) {
      return;
    }
    const isMulti = metrics?.length > 1;
    const opts = Object.assign({}, window.ApexOptionsBar);

    opts.series[0].data = metrics?.map((m) => {
      const value = CachedChannel[m] ? CachedChannel[m][1] : 0;
      return {
        x: m,
        y: value,
      };
    });

    const chart = new ApexCharts(elm.current, opts);
    chart.render();

    BusChannel[idx] = (date) => {
      metrics?.map((m) => {
        const value = CachedChannel[m] ? CachedChannel[m][1] : 0;
        if (!isMulti) {
          setValue(value);
        }
      });

      const options = {
        series: [
          {
            data: metrics?.map((m) => {
              const value = CachedChannel[m] ? CachedChannel[m][1] : 0;
              return {
                x: m,
                y: value,
              };
            }),
          },
        ],
      };
      chart.updateOptions(options, false, false);
    };

    return () => {
      delete BusChannel[idx];
    };
  }, [elm, metrics]);

  return html` <div class="col-md-4">
    <div class="box columnbox mt-4">
      <div class="header">
        <h3 class="title">${desc || metrics?.join(",")}</h3>
        ${metrics?.length === 1 &&
        html`<h2 class="subtitle">${value || "--"} ${unit ? unit : ""}</h2>`}
      </div>
      <div ref=${elm}></div>
    </div>
  </div>`;
};

function renderChart({ idx, chartType, meta }) {
  switch (chartType) {
    case "Bar":
      return html`<${BarChart}
        idx=${idx}
        metrics=${meta.metrics}
        desc=${meta.desc}
        unit=${meta.unit}
      />`;
    case "Line":
    default:
      return html`<${LineChart}
        idx=${idx}
        metrics=${meta.metrics}
        desc=${meta.desc}
        unit=${meta.unit}
      />`;
  }
}

function App() {
  const [charts, setCharts] = useState([]);
  useEffect(async () => {
    const chartres = await fetch("api/charts");
    const charts = await chartres.json();
    const metricres = await fetch("api/metrics");
    const metrics = await metricres.json();
    metrics.map((m) => {
      Metrics[m.key] = m;
    });
    setCharts(charts);

    const rawKeys = charts.map((m) => m.meta.metrics).flat();
    const keys = [...new Set(rawKeys)];
    const load = async () => {
      let now = new Date();
      let res = await fetch("api/metrics_value?keys=" + keys?.join(";"));
      let values = await res.json();
      console.log("loaded", values);
      values.map(({ key, value }) => {
        CachedChannel[key] = [now, value];
      });
      for (const idx in BusChannel) {
        BusChannel[idx](now);
      }
    };
    load();
    const interval = setInterval(load, 5000);

    return () => {
      clearInterval(interval);
    };
  }, []);

  return html` <div id="wrapper">
    <div class="content-area">
      <div class="container-fluid">
        <div class="main">
          <div class="row mt-4">
            ${charts.map((c, idx) =>
              renderChart({
                idx,
                chartType: c.type,
                meta: c.meta,
              })
            )}
          </div>
        </div>
      </div>
    </div>
  </div>`;
}

render(html`<${App} />`, document.body);
