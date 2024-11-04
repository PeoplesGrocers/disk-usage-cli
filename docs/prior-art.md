# Prior Art

This document surveys existing disk usage visualization and analysis tools. While this project as a proof of concept focuses specifically on .gitignore-aware visualization, many mature tools exist in this space that may better suit your needs.

## Command Line Tools

### dirstat-rs

https://github.com/scullionw/dirstat-rs

Claims to be much faster (On 4-core hyperthreaded cpu)

- 2X faster than du
- 4X faster than ncdu, dutree, dua, du-dust
- 6X faster than windirstat

Best visual design in my opinion of all of the Terminal UI's when showing relative size for list

### dua-cli

https://github.com/Byron/dua-cli

- Written in Rust for high performance
- Interactive TUI with real-time updates
- Parallel directory traversal
- Supports marking files/directories for deletion
- **Advantage over our tool**: Much more mature, better performance, interactive navigation
- **Missing**: No specific .gitignore awareness

### dust

https://github.com/bootandy/dust

- Written in Rust
- Tree-like output with visual indicators
- Smart terminal width handling
- **Advantage**: More intuitive output than du

### ncdu

https://dev.yorhel.nl/ncdu

- Classic, battle-tested tool
- Interactive TUI
- Very memory efficient
- File deletion capabilities
- **Advantage**: Rock solid stability, works everywhere, histogram visulization

### dutree

https://github.com/nachoparker/dutree
https://web.archive.org/web/20190205165745/https://ownyourbits.com/2018/03/25/analyze-disk-usage-with-dutree/

- coloured output, according to the LS_COLORS environment variable.
- display the file system tree
- ability to aggregate small files
- ability to exclude files or directories
- ability to compare different directories
- fast, written in Rust

### pdu

https://github.com/KSXGitHub/parallel-disk-usage

pdu is a CLI program that renders a graphical chart for disk usages of files and directories, it is an alternative to dust and dutree.

Most interesting thing to me is this tool has benchmarks unlike other tools.

## GUI Applications

### spaceman

https://github.com/salihgerdan/spaceman

- Fast scan and display, with the power of Rust, and gtk4 gpu rendering capabilities
- Uses the jwalk library as [dua-cli](https://github.com/Byron/dua-cli/) does, enabling multi-threaded scans
- Live display of scan results, no need to wait for the scan to complete
- Linux-first, but cross-platform
- **Advantage** tree map progressively loads

### GrandPerspective (macOS)

https://grandperspectiv.sourceforge.net/

- Beautiful treemap visualizations
- Real-time updates
- File filtering and focusing
- **Advantage**: Polished UI, great performance

### WinDirStat (Windows)

https://windirstat.net/

- Treemap visualization
- File type analysis
- Built-in cleanup tools
- **Advantage**: Deep Windows integration
- **Missing**: Cross-platform support

### Disk Inventory X (macOS)

http://www.derlien.com/

- Similar to WinDirStat for macOS
- Treemap visualization
- File type statistics
- **Advantage**: Native macOS experience
- **Missing**: Cross-platform support

### Disk Map: Visualize Disk Usage

https://fiplab.com/apps/disk-map-for-mac
https://apps.apple.com/us/app/disk-map-visualize-disk-usage/id715464874?mt=12

### SpaceSniffer

http://www.uderzo.it/main_products/space_sniffer/

Very interesting tagging idea. I'd quite like to mark some directories as won't fix, and then ignore them from future analysis.

> If you want to keep track of examined files you can tag them. Four colors are available. Just hover the mouse on a file and press CTRL+1 to tag it red. There are keys also for yellow, green and blue tagging. Use them as you wish. You can also filter on tags (example: :red will show only red tagges files, :all will show all tagged files and so on). You can also exclude tagged files (example: |:red will exclude all red tagged files)

### Disk Space Analyzer: Inspector

https://apps.apple.com/us/app/disk-space-analyzer-inspector/id446243721

- starburst visualization
- list largest files
- list largest directories
- Progress bar during the scanning process
- Navigating folders during the real-time scan
- Quick Look for the scanned items
- The Show in Finder option
- The list of 8 biggest items
- Outline view for navigating files
- All drives support

### TreeSize - Disk Usage

https://apps.apple.com/gb/app/treesize-disk-usage/id774815014

- color and group by mime type: audio, video, picture, document, archive, other
- Histogram list visualization
- Starburst visualization
- progressively loads visualization while scanning
- order by size, name or date.
- You can check the content of file by Quick look panel.
- You can see any file or folder in Finder by one click.
- You can open any file with proper application by one click.
- Added 'Open with TreeSize' in Service Menu.

### WhatSize

https://www.whatsizemac.com/

- The app uses the latest macOS technologies to stay up to date with any changes without having to re-scan everything.

## Integration Tools

### git-sizer

https://github.com/github/git-sizer

- Analyzes Git repository characteristics
- Identifies outsized objects and history
- **Advantage**: Deep Git integration
- **Missing**: General disk usage analysis, visualization
