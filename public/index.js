import {
  html,
  render,
  useState,
  useEffect,
  useRef,
} from "https://esm.sh/htm/preact/standalone";

const BusChannel = {};
const CachedChannel = {};

function Chart({ metric, desc, max_key, unit }) {
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

      if (max_key) {
        const maxv = CachedChannel[max_key] ? CachedChannel[max_key][1] : 0;
        setMaxValue(maxv)
        options.annotations = {
          yaxis: [
            {
              y: maxv,
              borderColor: "#FEB019",
              fillColor: "#FEB019",
              label: {
                style: {
                  color: "#000",
                  background: "#FEB019"
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
        <h2 class="subtitle">${value || "--"} ${maxValue ? " / " + maxValue : ""} ${unit || ""}</h2>
      </div>
      <div ref=${elm}></div>
    </div>
  </div>`;
}

function App() {
  const [metrics, setMetrics] = useState([]);
  useEffect(async () => {
    const res = await fetch("api/metrics");
    const metrics = await res.json();
    setMetrics(metrics);

    const keys = metrics.map((m) => m.key).join(";");
    const load = async () => {
      let now = new Date();
      let res = await fetch("api/metrics_value?keys=" + keys);
      let values = await res.json();
      console.log("loaded", values);
      values.map(({ key, value, unit, max_key }) => {
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
            ${metrics.map(
              (m) =>
                html`<${Chart}
                  metric=${m.key}
                  desc=${m.desc}
                  max_key=${m.max_key}
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
