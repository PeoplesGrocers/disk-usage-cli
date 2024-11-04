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

    // Update active segments
    const activeHTML = Array.from(this.activeSegments.entries())
      .map(([name, startTime]) => {
        const duration = performance.now() - startTime;
        return `<div class="${styles.activeSegment}">${name}: ${duration.toFixed(
          1
        )}ms</div>`;
      })
      .join("");

    let completedHTML = "";
    for (const [name, [duration]] of this.segments.entries()) {
      if (this.activeSegments.has(name)) {
        continue;
      }

      completedHTML += `<li><span class="${styles.segmentTime}">${formatTime(
        duration
      )}s</span> ${name}</li>\n`;
    }

    container.innerHTML = `
    <div class="${styles.timerSection}">
      <div class="${styles.timer}">${formatTime(elapsed)}s</div>
      <div id="${styles.activeSegments}">
        ${activeHTML}
      </div>
    </div>
    <div class="${styles.completedList}">
      <ul>
        ${completedHTML}
      </ul>
    </div>`;
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
