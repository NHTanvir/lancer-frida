//! lancer-frida — dynamic instrumentation toolkit (Lancer ecosystem).
//!
//! This is the shared library used by the main `lancer-frida` binary, every
//! companion binary in `src/bin/`, and the GUI. It follows the plan's Path A:
//! the real Frida engine is reached by delegating to the upstream `frida-*`
//! tools, while `lancer-frida` adds a unified CLI, a native GUI, an AI layer,
//! a bundled script library, and Lancer cross-tool integration.

pub mod commands;
pub mod core;
pub mod device;
pub mod exec;
pub mod script;
pub mod scripts;
pub mod selftest;

#[cfg(feature = "gui")]
pub mod gui;

/// Fallback when the crate is built with `--no-default-features` (CLI-only):
/// the GUI is not compiled in, so launching it is an error.
#[cfg(not(feature = "gui"))]
pub mod gui {
    pub fn launch() -> anyhow::Result<()> {
        anyhow::bail!(
            "this build has no GUI (compiled with --no-default-features) — \
             pass a subcommand, or rebuild with the default `gui` feature"
        )
    }
}

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lancer-frida", version, about = "Dynamic instrumentation toolkit (Lancer ecosystem)")]
#[command(long_about = "lancer-frida — a Rust controller + native GUI over the Frida dynamic \
instrumentation engine. Run with no subcommand to open the GUI; pass --cli or any subcommand \
to use the command-line surface.")]
pub struct Cli {
    /// Force the command-line surface (skip the GUI) — useful for CI/headless
    #[arg(long, global = true)]
    pub cli: bool,

    /// Run the upstream parity self-test matrix and print pass/fail per group
    #[arg(long, global = true)]
    pub self_test: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List available devices
    Devices,

    /// List running processes on a device
    Ps(DeviceArgs),

    /// Kill a process by name or PID
    Kill {
        #[command(flatten)]
        device: DeviceArgs,
        /// Process name or PID
        target: String,
    },

    /// Attach to a running process and inject a script
    Attach {
        #[command(flatten)]
        device: DeviceArgs,
        /// Process name or PID
        target: String,
        /// Load script from a file
        #[arg(short = 'l', long, value_name = "FILE")]
        script_file: Option<std::path::PathBuf>,
        /// Inline script code
        #[arg(short = 'e', long, value_name = "CODE")]
        eval: Option<String>,
    },

    /// Spawn a program and inject a script before first instruction
    Spawn {
        #[command(flatten)]
        device: DeviceArgs,
        /// Program to spawn
        program: String,
        /// Program arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
        /// Load script from a file
        #[arg(short = 'l', long, value_name = "FILE")]
        script_file: Option<std::path::PathBuf>,
        /// Inline script code
        #[arg(short = 'e', long, value_name = "CODE")]
        eval: Option<String>,
    },

    /// Trace function calls (like frida-trace)
    Trace {
        #[command(flatten)]
        device: DeviceArgs,
        /// Process name or PID
        target: String,
        /// Function pattern to include, e.g. 'open', '*ssl*', 'libssl!SSL_read'
        #[arg(short = 'i', long = "include", value_name = "PATTERN")]
        includes: Vec<String>,
    },

    /// Interactive JavaScript REPL in a process (like frida CLI)
    Repl {
        #[command(flatten)]
        device: DeviceArgs,
        /// Process name or PID
        target: String,
    },

    /// Browse, dump, or run bundled Frida scripts
    Scripts {
        #[command(subcommand)]
        action: ScriptsCmd,
    },

    /// List loaded Java/ObjC classes in a process
    Classes {
        #[command(flatten)]
        device: DeviceArgs,
        /// Process name or PID
        target: String,
        /// Filter class names (case-insensitive substring)
        #[arg(short = 'f', long, value_name = "FILTER")]
        filter: Option<String>,
        /// Platform: android (default) or ios
        #[arg(long, default_value = "android")]
        platform: String,
    },

    /// Memory operations: modules list, byte read, pattern scan
    Memory {
        #[command(subcommand)]
        action: MemoryCmd,
    },

    /// Discover security-relevant API surface in a process
    Discover {
        #[command(flatten)]
        device: DeviceArgs,
        /// Process name or PID
        target: String,
        /// Filter keyword (default: security-relevant patterns)
        #[arg(short = 'f', long, value_name = "FILTER")]
        filter: Option<String>,
        /// Ask Claude to rank the results (requires ANTHROPIC_API_KEY)
        #[arg(long)]
        ai: bool,
        /// Max results to display (0 = all)
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },

    /// Generate a Frida script using Claude AI
    Generate {
        /// Natural language description of what to instrument
        goal: String,
        /// Write script to file instead of stdout
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<std::path::PathBuf>,
    },

    /// AI-powered script diagnosis and fix
    Explain {
        /// Frida script file to diagnose (reads stdin if omitted)
        #[arg(value_name = "FILE")]
        script_file: Option<std::path::PathBuf>,
        /// Error message or crash output to provide as context
        #[arg(short = 'e', long, value_name = "ERROR")]
        error: Option<String>,
    },

    /// Summarize a captured trace/log into plain English (AI)
    Summarize {
        /// Trace/log file to summarize (reads stdin if omitted)
        #[arg(value_name = "FILE")]
        input: Option<std::path::PathBuf>,
    },

    /// Synthesize a bypass script from a described defense (AI)
    Synth {
        /// Description of the defense to bypass (e.g. "Cronet SSL pinning")
        description: String,
        /// Write script to file instead of stdout
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<std::path::PathBuf>,
    },

    /// Generate malformed fuzz seeds from observed inputs (AI)
    Fuzz {
        /// File of observed legitimate inputs (one per line; stdin if omitted)
        #[arg(value_name = "FILE")]
        input: Option<std::path::PathBuf>,
        /// Number of mutated seeds to request
        #[arg(short = 'n', long, default_value_t = 20)]
        count: usize,
    },

    /// Semantic search over the bundled + local script library (AI)
    Library {
        /// What you want a script to do
        query: String,
    },

    /// Suggest the next anti-detection evasion layer (AI)
    Evade {
        /// How the app detected you (or paste a detection log)
        symptom: String,
        /// Write script to file instead of stdout
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<std::path::PathBuf>,
    },

    /// Adapt a failing Java-hook script to a newer Android/ART version (AI)
    ArtUpdate {
        /// Target Android version (e.g. "16")
        android_version: String,
        /// Failing script file (reads stdin if omitted)
        #[arg(value_name = "FILE")]
        input: Option<std::path::PathBuf>,
    },

    /// Call an rpc.exports function in a loaded script
    Rpc {
        #[command(subcommand)]
        action: RpcCmd,
    },

    // ---- Delegated upstream tools (all args forwarded verbatim) ----
    /// Enumerate connected devices/backends (frida-ls-devices)
    LsDevices(Passthrough),
    /// Copy a file off the target device (frida-pull)
    Pull(Passthrough),
    /// Copy a file onto the target device (frida-push)
    Push(Passthrough),
    /// Remove a file on the target (frida-rm)
    Rm(Passthrough),
    /// List files on the target device (frida-ls)
    Ls(Passthrough),
    /// Create a directory on the target (frida-mkdir)
    Mkdir(Passthrough),
    /// Remove a directory on the target (frida-rmdir)
    Rmdir(Passthrough),
    /// Move/rename a file on the target (frida-mv)
    Mv(Passthrough),
    /// Sign and inject frida-gadget.so into an APK (frida-apk)
    Apk(Passthrough),
    /// Scaffold a new agent project (frida-create)
    Create(Passthrough),
    /// Compile a TS/ESM agent into a single bundle (frida-compile)
    Compile(Passthrough),
    /// Agent package manager (frida-pm / Frida.PackageManager)
    Pm(Passthrough),
    /// Connect a controller session to a frida-portal (frida-join)
    Join(Passthrough),
    /// Instruction-level Stalker trace (frida-itrace)
    Itrace(Passthrough),
    /// One-shot: load a script into a PID and exit (frida-inject)
    Inject(Passthrough),
    /// Multi-controller portal daemon (frida-portal)
    Portal(Passthrough),
    /// Statically patch an on-disk binary to load a Gum agent (gum-graft)
    Graft(Passthrough),
    /// Cross-platform syscall tracer (frida-strace)
    Strace(Passthrough),

    /// Run the upstream parity self-test matrix (same as --self-test)
    SelfTest,

    /// Launch the native GUI explicitly
    Gui,
}

/// Captures all trailing arguments and forwards them verbatim to an upstream
/// `frida-*` / `gum-graft` binary, giving full upstream-flag parity for free.
#[derive(Args, Clone, Default)]
pub struct Passthrough {
    /// Arguments forwarded verbatim to the upstream tool
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, value_name = "ARGS")]
    pub args: Vec<String>,
}

#[derive(Subcommand)]
pub enum RpcCmd {
    /// List rpc.exports functions exported by a script
    List {
        #[command(flatten)]
        device: DeviceArgs,
        /// Process name or PID
        target: String,
        /// Script file exporting rpc.exports
        #[arg(short = 'l', long, value_name = "FILE")]
        script_file: std::path::PathBuf,
    },
    /// Call an rpc.exports function
    Call {
        #[command(flatten)]
        device: DeviceArgs,
        /// Process name or PID
        target: String,
        /// Script file exporting rpc.exports
        #[arg(short = 'l', long, value_name = "FILE")]
        script_file: Option<std::path::PathBuf>,
        /// Name of the rpc.exports function to call
        function: String,
        /// Arguments (JSON or plain strings)
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum ScriptsCmd {
    /// List bundled scripts
    List,
    /// Print script source to stdout
    Dump {
        /// Script name
        name: String,
    },
    /// Run a bundled script against a process
    Run {
        #[command(flatten)]
        device: DeviceArgs,
        /// Process name or PID
        target: String,
        /// Script name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum MemoryCmd {
    /// List loaded modules
    Modules {
        #[command(flatten)]
        device: DeviceArgs,
        /// Process name or PID
        target: String,
    },
    /// Read bytes at an address
    Read {
        #[command(flatten)]
        device: DeviceArgs,
        /// Process name or PID
        target: String,
        /// Address (hex, e.g. 0x7fff1234)
        address: String,
        /// Number of bytes to read
        #[arg(default_value_t = 64)]
        length: usize,
    },
    /// Scan memory for a byte pattern
    Scan {
        #[command(flatten)]
        device: DeviceArgs,
        /// Process name or PID
        target: String,
        /// Pattern in Frida format, e.g. "de ad be ef ?? 01"
        pattern: String,
    },
}

/// Device selection flags (mutually exclusive)
#[derive(Args, Clone, Default)]
pub struct DeviceArgs {
    /// Use USB device
    #[arg(short = 'U', long, group = "device_select")]
    pub usb: bool,
    /// Use remote device (host:port)
    #[arg(short = 'R', long, value_name = "HOST", group = "device_select")]
    pub remote: Option<String>,
}

/// Initialise tracing once. Safe to call from every binary entry point.
pub fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}

/// Parse `std::env::args` and run. The entry point for the main binary.
pub fn run() -> anyhow::Result<()> {
    init_tracing();
    let cli = Cli::parse();

    // Global flags first.
    if cli.self_test {
        return selftest::run();
    }

    match cli.command {
        Some(command) => dispatch(command),
        None => {
            if cli.cli {
                // --cli with no subcommand: show help instead of opening the GUI.
                use clap::CommandFactory;
                Cli::command().print_help()?;
                println!();
                Ok(())
            } else {
                gui::launch()
            }
        }
    }
}

/// CLI-only entry point (the `lancer-frida-cli` companion): never opens the GUI.
pub fn run_cli() -> anyhow::Result<()> {
    init_tracing();
    let cli = Cli::parse();
    if cli.self_test {
        return selftest::run();
    }
    match cli.command {
        Some(command) => dispatch(command),
        None => {
            use clap::CommandFactory;
            Cli::command().print_help()?;
            println!();
            Ok(())
        }
    }
}

/// Entry point for a companion binary: behave as if `sub` were the first
/// argument, forwarding the rest of `std::env::args`.
pub fn run_subcommand(sub: &str) -> anyhow::Result<()> {
    init_tracing();
    let mut argv: Vec<std::ffi::OsString> =
        vec![std::ffi::OsString::from("lancer-frida"), std::ffi::OsString::from(sub)];
    argv.extend(std::env::args_os().skip(1));
    let cli = Cli::parse_from(argv);
    if cli.self_test {
        return selftest::run();
    }
    match cli.command {
        Some(command) => dispatch(command),
        None => {
            use clap::CommandFactory;
            Cli::command().print_help()?;
            println!();
            Ok(())
        }
    }
}

/// Dispatch a parsed subcommand.
pub fn dispatch(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Devices => commands::devices::run(),
        Commands::Ps(args) => commands::ps::run(&args),
        Commands::Kill { device, target } => commands::kill::run(&device, &target),
        Commands::Attach { device, target, script_file, eval } => {
            commands::attach::run(&device, &target, script_file.as_deref(), eval.as_deref())
        }
        Commands::Spawn { device, program, args, script_file, eval } => {
            commands::spawn::run(&device, &program, &args, script_file.as_deref(), eval.as_deref())
        }
        Commands::Trace { device, target, includes } => {
            commands::trace::run(&device, &target, &includes)
        }
        Commands::Repl { device, target } => commands::repl::run(&device, &target),
        Commands::Scripts { action } => match action {
            ScriptsCmd::List => {
                commands::scripts::list();
                Ok(())
            }
            ScriptsCmd::Dump { name } => commands::scripts::dump(&name),
            ScriptsCmd::Run { device, target, name } => {
                commands::scripts::run(&device, &target, &name)
            }
        },
        Commands::Classes { device, target, filter, platform } => {
            commands::classes::run(&device, &target, filter.as_deref(), &platform)
        }
        Commands::Memory { action } => match action {
            MemoryCmd::Modules { device, target } => commands::memory::modules(&device, &target),
            MemoryCmd::Read { device, target, address, length } => {
                commands::memory::read(&device, &target, &address, length)
            }
            MemoryCmd::Scan { device, target, pattern } => {
                commands::memory::scan(&device, &target, &pattern)
            }
        },
        Commands::Discover { device, target, filter, ai, limit } => {
            commands::discover::run(&device, &target, filter.as_deref(), ai, limit)
        }
        Commands::Generate { goal, output } => commands::generate::run(&goal, output.as_deref()),
        Commands::Explain { script_file, error } => {
            commands::explain::run(script_file.as_deref(), error.as_deref())
        }
        Commands::Summarize { input } => commands::ai_extra::summarize(input.as_deref()),
        Commands::Synth { description, output } => {
            commands::ai_extra::synth(&description, output.as_deref())
        }
        Commands::Fuzz { input, count } => commands::ai_extra::fuzz(input.as_deref(), count),
        Commands::Library { query } => commands::ai_extra::library(&query),
        Commands::Evade { symptom, output } => {
            commands::ai_extra::evade(&symptom, output.as_deref())
        }
        Commands::ArtUpdate { android_version, input } => {
            commands::ai_extra::artupdate(&android_version, input.as_deref())
        }
        Commands::Rpc { action } => match action {
            RpcCmd::List { device, target, script_file } => {
                commands::rpc::list(&device, &target, &script_file)
            }
            RpcCmd::Call { device, target, script_file, function, args } => {
                commands::rpc::call(&device, &target, script_file.as_deref(), &function, &args)
            }
        },
        Commands::LsDevices(p) => commands::tools::ls_devices(&p.args),
        Commands::Pull(p) => commands::tools::pull(&p.args),
        Commands::Push(p) => commands::tools::push(&p.args),
        Commands::Rm(p) => commands::tools::rm(&p.args),
        Commands::Ls(p) => commands::tools::ls(&p.args),
        Commands::Mkdir(p) => commands::tools::mkdir(&p.args),
        Commands::Rmdir(p) => commands::tools::rmdir(&p.args),
        Commands::Mv(p) => commands::tools::mv(&p.args),
        Commands::Apk(p) => commands::tools::apk(&p.args),
        Commands::Create(p) => commands::tools::create(&p.args),
        Commands::Compile(p) => commands::tools::compile(&p.args),
        Commands::Pm(p) => commands::tools::pm(&p.args),
        Commands::Join(p) => commands::tools::join(&p.args),
        Commands::Itrace(p) => commands::tools::itrace(&p.args),
        Commands::Inject(p) => commands::tools::inject(&p.args),
        Commands::Portal(p) => commands::tools::portal(&p.args),
        Commands::Graft(p) => commands::tools::graft(&p.args),
        Commands::Strace(p) => commands::tools::strace(&p.args),
        Commands::SelfTest => selftest::run(),
        Commands::Gui => gui::launch(),
    }
}
