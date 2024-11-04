const puppeteer = require("puppeteer");

async function captureScreenshots() {
  const browser = await puppeteer.launch();
  const page = await browser.newPage();

  // Set viewport size for consistent captures
  await page.setViewport({ width: 1200, height: 800 });

  try {
    // Load your local dev server
    await page.goto("http://localhost:8001");

    // Wait for visualization to load
    await page.waitForSelector(".visualization-container");

    // Take screenshots of each view
    await page.click('[data-view="treemap"]');
    // await page.waitForTimeout(1000); // Let animation complete
    await page.waitForSelector(".visualization-container", { visible: true });
    await page.screenshot({ path: "treemap.png" });

    await page.click('[data-view="starburst"]');
    await page.waitForTimeout(1000);
    await page.screenshot({ path: "starburst.png" });

    await page.click('[data-view="flamegraph"]');
    await page.waitForTimeout(1000);
    await page.screenshot({ path: "flamegraph.png" });
  } catch (error) {
    console.error("Error capturing screenshots:", error);
  } finally {
    await browser.close();
  }
}

captureScreenshots();
