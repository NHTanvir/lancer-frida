//! lancer-frida — dynamic instrumentation toolkit (Lancer ecosystem)

mod commands;
mod device;
mod script;
mod scripts;

use clap::{Args, Parser, Subcommand};
use frida::Frida;
use std::sync::LazyLock;

pub static FRIDA: LazyLock<Frida> = LazyLock::new(|| unsafe { Frida::obtain() });

#[derive(Parser)]
#[command(name = "lancer-frida", version, about = "Dynamic instrumentation toolkit")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
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

    /// Call an rpc.exports function in a loaded script
    Rpc {
        #[command(subcommand)]
        action: RpcCmd,
    },
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

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.command {
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
        Commands::Generate { goal, output } => {
            commands::generate::run(&goal, output.as_deref())
        }
        Commands::Explain { script_file, error } => {
            commands::explain::run(script_file.as_deref(), error.as_deref())
        }
        Commands::Rpc { action } => match action {
            RpcCmd::List { device, target, script_file } => {
                commands::rpc::list(&device, &target, &script_file)
            }
            RpcCmd::Call { device, target, script_file, function, args } => {
                commands::rpc::call(&device, &target, script_file.as_deref(), &function, &args)
            }
        },
    }
}
