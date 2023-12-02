window.Apex = {
  chart: {
    foreColor: "#fff",
    toolbar: {
      show: false,
    },
  },
  colors: ["#FCCF31", "#17ead9", "#f02fc2"],
  stroke: {
    curve: 'smooth'
  },
  dataLabels: {
    enabled: false,
  },
  grid: {
    borderColor: "#40475D",
  },
  xaxis: {
    axisTicks: {
      color: "#333",
    },
    axisBorder: {
      color: "#333",
    },
  },
  fill: {
    type: "gradient",
    gradient: {
      gradientToColors: ["#F55555", "#6078ea", "#6094ea"],
    },
  },
  tooltip: {
    theme: "dark",
    x: {
      formatter: function (val) {
        return moment(new Date(val)).format("HH:mm:ss");
      },
    },
  },
  yaxis: {
    decimalsInFloat: 2,
    opposite: true,
    labels: {
      offsetX: -10,
    },
  },
};

window.ApexOptionsColumn = {
  chart: {
    height: 250,
    type: "line",
    animations: {
      enabled: true,
    },
    toolbar: {
      show: false,
    },
    zoom: {
      enabled: false,
    },
  },
  dataLabels: {
    enabled: false,
  },
  stroke: {
    width: 2,
  },
  series: [
    {
      name: "--",
      data: [],
    },
  ],
  title: {
    text: "--",
    align: "left",
    style: {
      fontSize: "16px",
    },
  },
  subtitle: {
    text: '--',
    floating: true,
    align: 'right',
    offsetY: 0,
    style: {
      fontSize: '22px'
    }
  },
  fill: {
    type: "gradient",
    gradient: {
      shade: "dark",
      type: "vertical",
      shadeIntensity: 0.5,
      inverseColors: false,
      opacityFrom: 1,
      opacityTo: 0.8,
      stops: [0, 100],
    },
  },
  xaxis: {
    type: "datetime",
    range: 300000,
  },
  legend: {
    show: true,
  },
};
