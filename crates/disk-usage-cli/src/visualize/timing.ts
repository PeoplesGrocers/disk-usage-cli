import * as styles from "./timing.css";

export class RequestTimeline {
  private startTime: number;
  private segments: Map<string, [number, number]>; // [duration, startOffset]
  private activeSegments: Map<string, number>;

  constructor() {
    this.startTime = performance.now();
    this.segments = new Map();
    this.activeSegments = new Map();
  }

  startClock(name: string): boolean {
    if (this.activeSegments.has(name)) {
      return false;
    }
    this.activeSegments.set(name, performance.now());
    return true;
  }

  stopClock(name: string): number | null {
    const start = this.activeSegments.get(name);
    if (start === undefined) {
      return null;
    }

    this.activeSegments.delete(name);
    const now = performance.now();
    const duration = now - start;
    const startOffset = start - this.startTime;

    this.segments.set(name, [duration, startOffset]);
    return duration;
  }

  asServerTimingHeader(): string {
    return Array.from(this.segments.entries())
      .map(([name, [duration]]) => `${name};dur=${duration.toFixed(2)}`)
      .join(",");
  }

  totalDuration(): number {
    return performance.now() - this.startTime;
  }

  toString(): string {
    const times = Array.from(this.segments.entries())
      .map(
        ([name, [duration, offset]]) =>
          `${name}: ${duration.toFixed(2)}ms (started at +${offset.toFixed(
            2
          )}ms)`
      )
      .join("\n");
    return `Timeline (${this.totalDuration().toFixed(2)}ms total):\n${times}`;
  }

  updateDisplay(container: HTMLElement): void {
    // Update main timer
    const elapsed = this.totalDuration();

    let completedHTML = "";

    let i = 0;
    for (const [name, [duration]] of this.segments.entries()) {
      i++;
      if (this.activeSegments.has(name)) {
        continue;
      }

      completedHTML += `<li class=${styles.ruleItem}>
        <div class="${styles.numberCircle}">${i}</div>
        <span class="${styles.segmentName}">${name}</span>
        <span class="${styles.segmentTime}">${formatTime(duration)}s</span>
      </li>\n`;
    }

    // Update active segments
    for (const [name, startTime] of this.activeSegments.entries()) {
      const duration = performance.now() - startTime;
      completedHTML += `<li class="${styles.ruleItem} style="background:red">
        <div class="${styles.loader}"></div>
        <span class="${styles.segmentName}">${name}<span>
        <span class="${styles.segmentTime}">${duration.toFixed(1)}ms</span>
        </li>`;
    }

    container.innerHTML = `
    <div class="timerSection">
      <div class="timer">${formatTime(elapsed)}s</div>
    </div>
    <ul class=${styles.segmentList}>
      ${completedHTML}
    </ul>
    `;
  }
}

function formatTime(elapsed: number): string {
  if (elapsed < 1000) {
    return (elapsed / 1000).toFixed(2);
  } else if (elapsed < 10000) {
    return (elapsed / 1000).toFixed(1);
  } else {
    return Math.floor(elapsed / 1000).toString();
  }
}
