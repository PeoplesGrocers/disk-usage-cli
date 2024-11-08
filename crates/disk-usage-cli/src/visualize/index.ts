import * as styles from "./index.css";
import "./live-reload";
import { Metafile } from "./metafile";
import { showSummary } from "./summary";
import { createTreemap } from "./treemap";
import { createSunburst } from "./sunburst";
import { createFlame } from "./flame";
import { hideWhyFile } from "./whyfile";
import { showWarningsPanel } from "./warnings";
import { COLOR, updateColorMapping } from "./color";
import {
  darkModeListener,
  localStorageGetItem,
  localStorageSetItem,
} from "./helpers";

import { RequestTimeline } from "./timing";
export const timeline = new RequestTimeline();
const timelinePanel = document.getElementById("timelinePanel");

let startTime = Date.now();
let showAnimatedClock = true;

enum CHART {
  NONE,
  TREEMAP,
  SUNBURST,
  FLAME,
}

let startPanel = document.getElementById("startPanel") as HTMLDivElement;
let resultsPanel = document.getElementById("resultsPanel") as HTMLDivElement;
let chartPanel = document.getElementById("chartPanel") as HTMLDivElement;
let useTreemap = document.getElementById("useTreemap") as HTMLAnchorElement;
let useSunburst = document.getElementById("useSunburst") as HTMLAnchorElement;
let useFlame = document.getElementById("useFlame") as HTMLAnchorElement;
let chartMode = CHART.NONE;
export let colorMode = COLOR.NONE;

let isPlainObject = (value: any): boolean => {
  return (
    typeof value === "object" && value !== null && !(value instanceof Array)
  );
};

export let finishLoading = (json: string): void => {
  timeline.stopClock("load /metafile.json");
  timeline.startClock("Parse JSON");
  requestAnimationFrame(() => {
    // We wait one animation frame to allow rendering to catch up
    let metafile: Metafile = JSON.parse(json);
    timeline.stopClock("Parse JSON");

    let useChart = (use: CHART): void => {
      if (chartMode !== use) {
        if (chartMode === CHART.TREEMAP)
          useTreemap.classList.remove(styles.active);
        else if (chartMode === CHART.SUNBURST)
          useSunburst.classList.remove(styles.active);
        else if (chartMode === CHART.FLAME)
          useFlame.classList.remove(styles.active);

        chartMode = use;
        chartPanel.innerHTML = "";

        if (chartMode === CHART.TREEMAP) {
          chartPanel.append(createTreemap(metafile));
          useTreemap.classList.add(styles.active);
          localStorageSetItem("chart", "treemap");
        } else if (chartMode === CHART.SUNBURST) {
          chartPanel.append(createSunburst(metafile));
          useSunburst.classList.add(styles.active);
          localStorageSetItem("chart", "sunburst");
        } else if (chartMode === CHART.FLAME) {
          chartPanel.append(createFlame(metafile));
          useFlame.classList.add(styles.active);
          localStorageSetItem("chart", "flame");
        }
      }
    };

    let useColor = (use: COLOR): void => {
      if (colorMode !== use) {
        colorMode = use;
        timeline.startClock("updateColorMapping");
        updateColorMapping(metafile, colorMode);
        timeline.stopClock("updateColorMapping");
        console.info(timeline.toString());
      }
    };

    if (
      !isPlainObject(metafile) ||
      !isPlainObject(metafile.inputs) ||
      !isPlainObject(metafile.outputs)
    ) {
      throw new Error("Invalid metafile format");
    }

    // Only at the very end do we stop the animated clock
    startPanel.style.display = "none";

    resultsPanel.style.display = "block";
    useTreemap.onclick = () => useChart(CHART.TREEMAP);
    useSunburst.onclick = () => useChart(CHART.SUNBURST);
    useFlame.onclick = () => useChart(CHART.FLAME);

    chartMode = CHART.NONE;
    colorMode = COLOR.NONE;
    showSummary(metafile, () =>
      useColor(colorMode === COLOR.DIRECTORY ? COLOR.FORMAT : COLOR.DIRECTORY)
    );
    showWarningsPanel(metafile);
    hideWhyFile();

    timeline.startClock(
      `analyze ${
        localStorageGetItem("chart") === "flame"
          ? "flame"
          : localStorageGetItem("chart") === "sunburst"
          ? "sunburst"
          : "treemap"
      }`
    );

    requestAnimationFrame(() => {
      useChart(
        localStorageGetItem("chart") === "flame"
          ? CHART.FLAME
          : localStorageGetItem("chart") === "sunburst"
          ? CHART.SUNBURST
          : CHART.TREEMAP
      );
      if (timelinePanel) {
        timeline.updateDisplay(timelinePanel);
      }
      requestAnimationFrame(() => {
        useColor(COLOR.DIRECTORY);

        showAnimatedClock = false;
        if (timelinePanel) {
          timeline.updateDisplay(timelinePanel);
        }
      });
    });
  });
};

let docElemDataset = document.documentElement.dataset;
let updateTheme = () => {
  // Keep the dark/light mode theme up to date with the rest of the site
  docElemDataset.theme = localStorageGetItem("theme") + "";
  if (darkModeListener) darkModeListener();
};

updateTheme();
window.addEventListener("storage", updateTheme);

timeline.startClock("load /metafile.json");
fetch("/metafile.json")
  .then((r) => r.text())
  .then(finishLoading)
  .catch((error) => {
    // statusDisplay.textContent = 'Error loading file';
    showAnimatedClock = false;
    console.error("Stopping the loading indicator");
    console.error(error);
    // cancelAnimationFrame(animationId);
  });

function updateTimer() {
  if (!showAnimatedClock) return;

  const elapsed = Date.now() - startTime;
  if (timelinePanel) {
    timeline.updateDisplay(timelinePanel);
    requestAnimationFrame(updateTimer);
  }
}

updateTimer();
