import {
  html,
  render,
  useState,
  useEffect,
  useRef,
} from "https://esm.sh/htm/preact/standalone";

const BusChannel = {};
const CachedChannel = {};

function Chart({ metric, desc, chartType, meta, unit }) {
  const elm = useRef(null);
  const [value, setValue] = useState();
  const [maxValue, setMaxValue] = useState();
  useEffect(() => {
    if (!elm) {
      return;
    }
    const data = CachedChannel[metric] ? [CachedChannel[metric]] : [];
    const opts = Object.assign({}, window.ApexOptionsColumn);
    opts.title.text = "";
    opts.subtitle.text = "";
    switch (chartType) {
      case "Line":
        opts.chart.type = "line";
        break;
      case "Bar":
        opts.chart.type = "bar";
        break;
      default:
        opts.chart.type = "line";
    }

    opts.series[0].name = desc || metric;
    opts.series[0].data = data;

    const chart = new ApexCharts(elm.current, opts);
    chart.render();
    BusChannel[metric] = (date, value) => {
      data.push([date, value]);
      setValue(value);

      if (data.length > 100) {
        data.shift();
      }

      const options = {
        series: [
          {
            data,
          },
        ],
      };

      if (meta?.max_metric) {
        const max_key = meta?.max_metric;
        const maxv = CachedChannel[max_key] ? CachedChannel[max_key][1] : 0;
        setMaxValue(maxv);
        options.annotations = {
          yaxis: [
            {
              y: maxv,
              borderColor: "#FEB019",
              fillColor: "#FEB019",
              opacity: 1,
              label: {
                style: {
                  color: "#000",
                  background: "#FEB019",
                },

                text: max_key + ": " + maxv,
              },
            },
          ],
        };
      }

      chart.updateOptions(options, false, false);
    };

    return () => {
      delete BusChannel[metric];
    };
  }, [elm]);

  return html` <div class="col-md-4">
    <div class="box columnbox mt-4">
      <div class="header">
        <h3 class="title">${desc || metric}</h3>
        <h2 class="subtitle">
          ${value || "--"} ${maxValue ? " / " + maxValue : ""} ${unit || ""}
        </h2>
      </div>
      <div ref=${elm}></div>
    </div>
  </div>`;
}

function App() {
  const [charts, setCharts] = useState([]);
  useEffect(async () => {
    const res = await fetch("api/charts");
    const charts = await res.json();
    setCharts(charts);

    const maxKeys = charts
      .filter((m) => m.chart_type?.meta?.max_metric)
      .map((m) => m.chart_type?.meta?.max_metric);
    const keys = charts.map((m) => m.key);
    const load = async () => {
      let now = new Date();
      let res = await fetch(
        "api/metrics_value?keys=" + [...keys, ...maxKeys].join(";")
      );
      let values = await res.json();
      console.log("loaded", values);
      values.map(({ key, value }) => {
        if (BusChannel[key]) {
          BusChannel[key](now, value);
        }
        CachedChannel[key] = [now, value];
      });
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
            ${charts.map(
              (m) =>
                html`<${Chart}
                  metric=${m.key}
                  desc=${m.desc}
                  chartType=${m.chart_type.type}
                  meta=${m.chart_type.meta}
                  unit=${m.unit}
                />`
            )}
          </div>
        </div>
      </div>
    </div>
  </div>`;
}

render(html`<${App} />`, document.body);
