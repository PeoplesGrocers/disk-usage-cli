// showing disk usage with .gitignore awareness
// Copyright (C) 2024 Peoples Grocers LLC
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
use clap::{Arg, ArgAction};
use ignore::WalkBuilder;
use indexmap::IndexMap;
use indicatif::{ProgressBar, ProgressStyle};
use num_format::{Locale, SystemLocale, ToFormattedString};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

mod visualize;

/// Format a number with locale-aware thousands separators
fn format_number(num: u64) -> String {
    // Try to use system locale, fall back to US English locale if it fails
    match SystemLocale::default() {
        Ok(locale) => num.to_formatted_string(&locale),
        Err(_) => {
            // Fallback: US English locale with comma separators
            num.to_formatted_string(&Locale::en)
        }
    }
}

fn main() {
    let mut command = clap::Command::new("duh")
        .version("0.1.0")
        .author("nobody <nobody@localhost>")
        .about("Displays disk usage statistics for ignored files")
        .long_about("The duh utility displays the file system block usage for each file argument and for each directory in the file hierarchy rooted in each directory argument. If no file is specified, the block usage of the hierarchy rooted in the current directory is displayed. By default duh will respect gitignore rules and automatically skip hidden files/directories.")
        .disable_help_flag(true)
        .arg(Arg::new("help").long("help").action(ArgAction::Help));
    #[cfg(unix)]
    {
        command = command
        .arg(
            Arg::new("apparent")
                .short('A')
                .long("apparent")
                .action(ArgAction::SetTrue)
                .help("Display the apparent size instead of the disk usage. This can be helpful when operating on compressed volumes or sparse files."),
        );
    }
    command = command.arg(
            Arg::new("hidden")
                .short('H')
                .long("hidden")
                .help("Include hidden files and directories in the count"),
        )
        .arg(
            Arg::new("show-files")
                .short('a')
                .action(ArgAction::SetTrue)
                .help("Display an entry for each file in a file hierarchy.")
        )
        .arg(
            Arg::new("max-depth")
                .short('d')
                .long("depth")
                .value_parser(clap::value_parser!(u64))
                .action(ArgAction::Set)
                .help("Display an entry for all files and directories depth directories deep."),
        )
        .arg(
            Arg::new("direct-size")
                .long("direct-size")
                .action(ArgAction::SetTrue)
                .help("Only calculate size using direct contents of the directory. This can be helpful when trying to find what is using all that disk space"),
        )
        .arg(
            Arg::new("web")
                .long("web")
                .action(ArgAction::SetTrue)
                .help("Start a web server to visualize disk usage with interactive flamegraph, starburst, and treemap"),
        )
        .arg(
            Arg::new("open")
                .long("open")
                .action(ArgAction::SetTrue)
                .help("Start web server and open visualization in your default browser"),
        )
        .arg(
            Arg::new("human-readable")
                .short('h')
                .action(ArgAction::SetTrue)
                .help("Print sizes in human readable format (e.g., 1K 234M 2G)"),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                // I used to have the default value as "ignored" because I
                // thought I would want to see only the ignored files, but in
                // practice I was always interested in seeing the breakdown.
                // Also it takes the same amount of time so we aren't saving
                // anything.
                .default_value("du")
                .value_parser(clap::value_parser!(Mode))
                .action(ArgAction::Set)
                .help(""),
        )
        .arg(
            Arg::new("summary")
                .short('s')
                .action(ArgAction::SetTrue)
                .conflicts_with("max-depth")
                .help("Display an entry for each specified file.  (Equivalent to -d 0)"),
        )
        .arg(
            Arg::new("PATH")
                .default_value("./")
                .action(ArgAction::Append)
                .long_help("")
                .help("Specify the paths to analyze"),
        );
    let matches = command.get_matches();

    // TODO change this into get_many
    let mut paths: Vec<String> = matches
        .get_many::<String>("PATH")
        .unwrap_or_default()
        .map(|v| v.to_owned())
        .collect::<Vec<_>>();

    if paths.is_empty() {
        paths.push("./".to_owned())
    }

    let config = Config::parse(matches);

    if (config.open || config.web) && paths.len() > 1 {
        eprintln!("Cannot open multiple directories in web mode");
        std::process::exit(1);
    }

    for path in paths {
        process_directory(path.as_str(), &config);
    }
    //self::visualize::view_in_browser();
}

struct Config {
    include_hidden: bool,
    max_depth: Option<u64>,
    human_readable: bool,
    use_apparent_size: bool,
    mode: Mode,
    use_recursive_size: bool,
    show_only_directories: bool,
    web: bool,
    open: bool,
}

impl Config {
    fn parse(matches: clap::ArgMatches) -> Self {
        let human_readable = matches.get_flag("human-readable");
        let include_hidden = matches.contains_id("hidden");
        let use_apparent_size = matches.get_flag("apparent");
        let direct_size = matches.get_flag("direct-size");
        let open = matches.get_flag("open");
        let web = matches.get_flag("web");

        let mode: Mode = matches
            .get_one::<Mode>("mode")
            .copied()
            .unwrap_or(Mode::Ignored);
        let show_only_directories: bool = matches
            .get_one::<bool>("show-files")
            .map(|show_files| !show_files)
            .unwrap_or(mode == Mode::Du);

        // I am relying on clap::Arg::conflicts_with to prevent -s and --depth from being used
        // simultatenously
        let mut max_depth: Option<u64> = matches.get_one::<u64>("max-depth").copied();
        if matches.get_flag("summary") {
            max_depth = Some(0)
        }

        Self {
            include_hidden,
            max_depth,
            human_readable,
            use_apparent_size,
            mode,
            use_recursive_size: !direct_size,
            show_only_directories,
            web: web || open,
            open,
        }
    }
}

#[derive(Copy, PartialEq, Eq, Clone, clap::ValueEnum)]
enum Mode {
    Du,
    Ignored,
    NotIgnored,
}

impl Mode {
    fn description(&self) -> &'static str {
        match self {
            Mode::Du => "all files (both ignored and not ignored)",
            Mode::Ignored => "only files that are ignored by .gitignore rules",
            Mode::NotIgnored => "only files that are NOT ignored by .gitignore rules",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Mode::Du => "du",
            Mode::Ignored => "ignored",
            Mode::NotIgnored => "not-ignored",
        }
    }
}

fn process_directory(path: &str, config: &Config) {
    // Explain what mode is being used
    eprintln!("Mode: '{}' - analyzing {}", config.mode.name(), config.mode.description());
    eprintln!("Note: Progress bar shows ALL files visited, but results will only include files matching the selected mode.\n");

    let walker = WalkBuilder::new(path).hidden(config.include_hidden).filter_entry(|entry| {
        // It seems that .git directories are not automatically ignored. Weird.
        entry.file_name().to_str().map(|s| { s != ".git"}).unwrap_or(true)
    }).build();

    let mut groups: IndexMap<PathBuf, (u64, u64, bool)> = IndexMap::new();
    // We also to keep track of the largest value in this hashmap so we can format the output We
    // will print the size of each
    let mut max_size: u64 = 0;

    //let starting_absolute_path = std::env::current_dir()
    //    .unwrap()
    //    .join(Path::new(path))
    //    .canonicalize()
    //    .unwrap();

    // Setup progress bar with spinner style
    let pb = ProgressBar::new_spinner();
    let spinner_style = ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    pb.set_style(spinner_style);
    pb.set_message("Scanning files... (0 files)");

    // Throttling mechanism - update at most 10 times per second
    let mut file_count = 0u64;
    let mut ignored_count = 0u64; // Count of ignored files (for du mode breakdown)
    let mut not_ignored_count = 0u64; // Count of not ignored files (for du mode breakdown)
    let mut last_update = Instant::now();
    let update_interval = Duration::from_millis(100); // 100ms = 10 times per second

    for result in walker {
        match result {
            Err(_) => continue,
            Ok(dent) => {
                // Update file count and progress bar (throttled)
                file_count += 1;
                
                // Track ignored vs not-ignored counts for du mode breakdown
                if dent.ignored {
                    ignored_count += 1;
                } else {
                    not_ignored_count += 1;
                }
                
                let now = Instant::now();
                if now.duration_since(last_update) >= update_interval {
                    pb.set_message(format!("Scanning files... ({} files)", format_number(file_count)));
                    pb.tick();
                    last_update = now;
                }

                #[cfg(unix)]
                use std::os::unix::fs::MetadataExt;
                let size = {
                    #[cfg(unix)]
                    match dent.metadata() {
                        Err(_) => continue,
                        Ok(metadata) => {
                            if config.use_apparent_size {
                                metadata.len()
                            } else {
                                metadata.blocks() * 512
                            }
                        }
                    }
                    #[cfg(not(unix))]
                    match dent.metadata() {
                        Err(_) => continue,
                        Ok(metadata) => metadata.len(),
                    }
                };

                // I want to take up to max_depth components from the directory portion of a
                // filepath, but not include the filename.
                struct FileComponentIter<'a> {
                    iter: std::iter::Peekable<std::path::Iter<'a>>,
                    current_depth: u64,
                    max_depth: Option<u64>,
                }

                impl<'a> Iterator for FileComponentIter<'a> {
                    type Item = &'a std::ffi::OsStr;

                    fn next(&mut self) -> Option<Self::Item> {
                        if self.iter.peek().is_none() || Some(self.current_depth) == self.max_depth
                        {
                            None
                        } else {
                            self.current_depth += 1;
                            self.iter.next()
                        }
                    }
                }

                match dent.file_type() {
                    None => (),
                    Some(ft) => {
                        if let Ok(relative_path) = dent.path().strip_prefix(path) {
                            let should_process = config.mode == Mode::Du
                                || (config.mode == Mode::Ignored && dent.ignored)
                                || (config.mode == Mode::NotIgnored && !dent.ignored);

                            if !should_process {
                                continue;
                            }

                            let it: Box<dyn Iterator<Item = &'_ std::ffi::OsStr>> = if ft.is_file()
                            {
                                Box::new(FileComponentIter {
                                    iter: relative_path.iter().peekable(),
                                    current_depth: 0,
                                    max_depth: config.max_depth,
                                })
                            } else {
                                if let Some(max_depth) = config.max_depth {
                                    Box::new(relative_path.iter().take(max_depth as usize))
                                } else {
                                    Box::new(relative_path.iter())
                                }
                            };
                            let mut key = PathBuf::new();
                            for component in it {
                                if config.use_recursive_size {
                                    if let Some((ref mut not_ignored_acc, ref mut ignored_acc, _)) =
                                        groups.get_mut(&key)
                                    {
                                        if dent.ignored {
                                            *ignored_acc += size;
                                        } else {
                                            *not_ignored_acc += size;
                                        }
                                        max_size = std::cmp::max(
                                            max_size,
                                            *ignored_acc + *not_ignored_acc,
                                        );
                                    } else {
                                        let (ref mut not_ignored_acc, ref mut ignored_acc, _) =
                                            groups.entry(key.clone()).or_insert((
                                                0,
                                                0,
                                                ft.is_file(),
                                            ));
                                        if dent.ignored {
                                            *ignored_acc += size;
                                        } else {
                                            *not_ignored_acc += size;
                                        }
                                        max_size = std::cmp::max(
                                            max_size,
                                            *ignored_acc + *not_ignored_acc,
                                        );
                                    }
                                }
                                key.push(component);
                            }
                            let (ref mut not_ignored_acc, ref mut ignored_acc, _) =
                                groups.entry(key.clone()).or_insert((0, 0, ft.is_file()));
                            if dent.ignored {
                                *ignored_acc += size;
                            } else {
                                *not_ignored_acc += size;
                            }
                            max_size = std::cmp::max(max_size, *ignored_acc + *not_ignored_acc);
                        } else {
                            eprintln!("failed to strip prefix")
                        }
                    }
                }
            }
        }
    }

    // Finish progress bar with final file count
    let final_message = match config.mode {
        Mode::Du => {
            format!("Visited {} files, {} ignored, {} not ignored", 
                format_number(file_count),
                format_number(ignored_count),
                format_number(not_ignored_count)
            )
        },
        Mode::Ignored => {
            format!("Visited {} files, {} ignored", 
                format_number(file_count), 
                format_number(ignored_count)
            )
        },
        Mode::NotIgnored => {
            format!("Visited {} files, {} not ignored", 
                format_number(file_count), 
                format_number(not_ignored_count)
            )
        }
    };
    pb.finish_with_message(final_message);

    let mut pairs: Vec<(PathBuf, (u64, u64, bool))> = groups.into_iter().collect();

    if config.web {
        self::visualize::view_in_browser(&pairs, config.open);
        return;
    }

    pairs.sort_by(|a, b| {
        let mut xs = a.0.iter();
        let mut ys = b.0.iter();
        use std::cmp::Ordering;
        loop {
            match (xs.next(), ys.next()) {
                (Some(x), Some(y)) => match x.cmp(&y) {
                    Ordering::Equal => continue,
                    non_eq => return non_eq,
                },
                (Some(_), None) => return Ordering::Less,
                (None, Some(_)) => return Ordering::Greater,
                (None, None) => return Ordering::Equal,
            }
        }
    });

    for (suffix, (not_ignored_size, ignored_size, is_file)) in pairs {
        if config.show_only_directories && is_file {
            continue;
        }

        let size = match config.mode {
            Mode::Du => not_ignored_size + ignored_size,
            Mode::Ignored => ignored_size,
            Mode::NotIgnored => not_ignored_size,
        };

        if config.human_readable {
            eprintln!(
                "{} {}",
                format_human_readable(size),
                Path::new(path).join(suffix).display()
            );
        } else {
            let first_column_width = (max_size / 512).to_string().len();
            eprintln!(
                "{:>first_column_width$} {}",
                size / 512,
                Path::new(path).join(suffix).display(),
                first_column_width = first_column_width,
            );
        }
    }
}

fn format_human_readable(size: u64) -> String {
    let sizes = ["B", "K", "M", "G", "T", "P", "E"];
    let factor = 1024u64;
    let mut size = size as f64;
    let mut i = 0;
    while size >= factor as f64 && i < sizes.len() - 1 {
        size /= factor as f64;
        i += 1;
    }
    if size < 10.0 {
        return format!("{:4.1}{}", size, sizes[i]);
    }
    format!("{:4.0}{}", size, sizes[i])
}
