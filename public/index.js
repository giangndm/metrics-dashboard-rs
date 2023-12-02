import { html, render, useState, useEffect, useRef } from 'https://esm.sh/htm/preact/standalone'

const BusChannel = {};
const CachedChannel = {};

function Chart ({ metric, desc }) {
    const elm = useRef(null);
    useEffect(() => {
        if (!elm) {
            return;
        }
        const data = CachedChannel[metric] ? [CachedChannel[metric]] : [];
        const opts = Object.assign({}, window.ApexOptionsColumn);
        opts.title.text = desc;
        opts.subtitle.text = data[0] ? data[0][1] : '--';
        opts.series[0].name = desc;
        opts.series[0].data = data;
        
        const chart = new ApexCharts(elm.current, opts);
        chart.render();
        BusChannel[metric] = (date, value) => {
            data.push([date, value]);
            if (data.length > 100) {
                data.shift();
            }
            chart.updateOptions({
                series: [{
                  data
                }],
                subtitle: {
                  text: value,
                }
            }, false, false);
        }

        return () => {
            delete BusChannel[metric];
        }
    }, [elm])

    return html`
    <div class="col-md-4">
        <div class="box columnbox mt-4">
            <div ref=${elm}></div>
        </div>
    </div>`;
}

function App () {
    const [metrics, setMetrics] = useState([]);
    useEffect(async () => {
        const res = await fetch('api/metrics');
        const metrics = await res.json();
        setMetrics(metrics);

        const keys = metrics.map((m) => m.key).join(';');
        const load = async () => {
            let now = new Date();
            let res = await fetch('api/metrics_value?keys=' + keys);
            let values = await res.json();
            console.log('loaded', values);
            values.map(({key, value}) => {
                if (BusChannel[key]) {
                    BusChannel[key](now, value);
                }
                CachedChannel[key] = [now, value];
            });
        }
        load();
        const interval = setInterval(load, 5000);

        return () => {
            clearInterval(interval);
        }
    }, []);

    return html`
    <div id="wrapper">
		<div class="content-area">
			<div class="container-fluid">
				<div class="main">
                    <div class="row mt-4">
                    ${metrics.map((m) => html`<${Chart} metric=${m.key} desc=${m.desc}/>`)}
                    </div>
                </div>
            </div>
        </div>
    </div>`;
}

render(html`<${App}/>`, document.body);