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

// import * as hack from "./timing.css";
// let loader = document.createElement("div");
// loader.style.float = "right";
// loader.classList.add(hack.loader);
// document.body.insertBefore(loader, document.body.firstChild);

import { RequestTimeline } from "./timing";
export const timeline = new RequestTimeline();

const MORPH_PANEL_DELAY = 500;
// I decided to pretend these elements would always exist to avoid writing out null checks that
// would make the app bundle size larger.
const panel = document.getElementById("timelinePanel")!;
const drawer = document.getElementById("timeline-drawer")!;
const drawerToggle = document.getElementById("timeline-drawer-toggle")!;
const timelineContent = document.getElementById("timeline-drawer-content")!;

timeline.updateDisplay(timelineContent);

function animateHeight() {
  const startHeight = panel.offsetHeight;
  panel.style.height = startHeight + "px";
  panel.style.transition = "height 0.4s cubic-bezier(0.4, 0, 0.2, 1)";

  requestAnimationFrame(() => {
    // Triggering the reflow is critical for the transition to work
    void panel.offsetHeight;
    panel.style.height = "0px";
  });
}

function morphPanelToButton() {
  // animateHeight();
  requestAnimationFrame(() => {
    drawer.classList.add("shrink");
    panel.classList.add("shrink");
    drawerToggle.classList.remove("startHidden");

    setTimeout(() => {
      panel.removeAttribute("open");
      drawer.classList.remove("shrink");
      panel.classList.remove("shrink");

      panel.style.height = "";
      panel.style.transition = ""; // clean up the transition
    }, 400);
  });
}

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
  timeline.updateDisplay(timelineContent);
  debugger;
  requestAnimationFrame(() => {
    // We wait one animation frame to allow rendering to catch up
    let metafile: Metafile = JSON.parse(json);
    timeline.stopClock("Parse JSON");
    timeline.updateDisplay(timelineContent);

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
        updateColorMapping(metafile, colorMode);
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
    timeline.updateDisplay(timelineContent);

    requestAnimationFrame(() => {
      useChart(
        localStorageGetItem("chart") === "flame"
          ? CHART.FLAME
          : localStorageGetItem("chart") === "sunburst"
          ? CHART.SUNBURST
          : CHART.TREEMAP
      );

      timeline.updateDisplay(timelineContent);

      requestAnimationFrame(() => {
        timeline.startClock("updateColorMapping");
        timeline.updateDisplay(timelineContent);
        requestAnimationFrame(() => {
          useColor(COLOR.DIRECTORY);
          timeline.stopClock("updateColorMapping");
          console.info(timeline.toString());
          timeline.updateDisplay(timelineContent);

          showAnimatedClock = false;
          // Once the app is interactive, morph the timeline panel into a floating button in the top left
          setTimeout(() => {
            morphPanelToButton();
          }, MORPH_PANEL_DELAY);
        });
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

requestAnimationFrame(() => {
  timeline.startClock("load /metafile.json");
  timeline.updateDisplay(timelineContent);
  // This fetch needs to be in the next animation frame to avoid blocking the initial render
  fetch("/metafile.json")
    .then((r) => r.text())
    .then(finishLoading)
    .catch((error) => {
      // statusDisplay.textContent = 'Error loading file';
      showAnimatedClock = false;
      timeline.updateDisplay(timelineContent);
      console.error("Stopping the loading indicator");
      console.error(error);
      // cancelAnimationFrame(animationId);
    });
});

function updateTimer() {
  if (!showAnimatedClock) return;

  timeline.updateDisplay(timelineContent);
  requestAnimationFrame(updateTimer);
}

updateTimer();
