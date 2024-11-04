# Design Documentation

## Design Decisions

1. **Embedded Web App**: Rather than requiring users to install a separate GUI application, we embed a small web visualization directly in the binary. This provides a seamless experience while keeping the tool self-contained.

2. **Multiple Visualization Types**: Each visualization type (treemap, starburst, flamegraph) offers different insights:

   - Treemap: I like seeing how unexpectedly large files "pop" out.
   - Starburst: I like to see the names of tiny files and directories too. I'll use it to clean up tiny hidden files.
   - Flamegraph: Ctrl+Scroll to zoom is way easier to implement for 1D than for 2D treemap.

3. **Standard Library HTTP Server**: Using std::net keeps our dependencies minimal while providing the necessary functionality for serving the visualization.

## Implementation Choices

### Web Visualization

I built the visualization component by modifying the esbuild bundle visualizer. A few things made this an obvious choice:

1. I had just used it the previous week and noticed how clean the implementation was
2. The entire webapp would only add about 40KB when embedded in the CLI
3. It already implemented the key feature I needed - toggling between two categories of files
4. Evan Wallace's code is notably efficient

I suspected I could adapt it for visualizing ignored vs non-ignored file structures with minimal changes to the core visualization logic.

The main tradeoffs were:

- Pro: Very small payload size
- Pro: Could reuse existing, well-tested visualization code
- Pro: Already handled the exact category-switching interaction I wanted
- Con: Had to embed a webapp in a CLI tool (but the size made this acceptable)

### Binary Size Impact

The tool starts at 1.7MB stripped (2.0MB unstripped), so I tracked size impact carefully but wasn't obsessive about it. Here's what each feature added:

| Change                                                 | Size      | Delta  | Stripped  |
| ------------------------------------------------------ | --------- | ------ | --------- |
| Baseline: Core functionality                           | 2,074,136 | -      | 1,761,136 |
| Add serde_json (export disk usage as esbuild metafile) | 2,110,600 | 36,464 | 1,794,240 |
| Simple HTTP server (std::net)                          | 2,127,512 | 16,912 | 1,811,312 |
| Browser auto-open functionality (UX improvement)       | 2,169,544 | 42,032 | 1,846,032 |
| Embedded web app                                       |           | 39,242 |           |

Adding JSON export with serde_json was the first big jump. The basic HTTP server (using std::net) was surprisingly cheap.

The browser auto-open feature saves me ~2 seconds per run - I measured this by logging duration between when the URL was printed to stdout and when the webapp made its first API request. Seeing the results 2 seconds faster was worth extra 42KB to me. If your terminal makes links clickable then obviously the tra

Even with terminals that support clickable URLs, auto-open still improves UX. Since the disk usage scan can take >10 seconds, I typically start the command and switch focus elsewhere. It's like cargo doc --open - you want the browser tab to just appear when the work is done, catching my eye on my second monitor. Without auto-open, there's still reaction time and context switching overhead that adds 500ms+ delays, even with clickable links.

There's clearly room to optimize - starting at 1.7MB for core functionality suggests we could probably slim things down quite a bit. But since no single feature added more than 42KB, I focused on shipping useful functionality first.

### Performance Characteristics

Measured on a Apple M2 Pro - 32GB - 2023

#### Small Directory Tree (55,717 files, depth 6)

| Operation        | Duration |
| ---------------- | -------- |
| File reading     | 23ms     |
| JSON Parsing     | 65ms     |
| analyze Treemap  | 143ms    |
| analyze Sunburst | 173ms    |
| analyze Flame    | 164ms    |

#### Large Directory Tree (1.7M files, 589MB, depth 10)

| Operation        | Duration |
| ---------------- | -------- |
| File reading     | 860ms    |
| JSON Parsing     | 3,195ms  |
| analyze Treemap  | 8,664ms  |
| analyze Sunburst | 12,017ms |
| analyze Flame    | 8,476ms  |

Note: Color mapping updates take ~1.8-2.2 seconds for the 1.7M entry case.
