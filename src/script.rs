use frida::{Message, ScriptHandler};
use std::path::Path;
use std::sync::mpsc::Sender;

pub struct LogHandler;

impl ScriptHandler for LogHandler {
    fn on_message(&mut self, message: Message, _data: Option<Vec<u8>>) {
        match message {
            Message::Log(log) => println!("[{:?}] {}", log.level, log.payload),
            Message::Send(send) => println!("[send] {:?}", send.payload),
            Message::Error(err) => eprintln!(
                "[error] {} at {}:{}:{}",
                err.description, err.file_name, err.line_number, err.column_number
            ),
            Message::Other(val) => println!("[msg] {:?}", val),
        }
    }
}

pub struct ChannelHandler {
    pub tx: Sender<serde_json::Value>,
}

impl ScriptHandler for ChannelHandler {
    fn on_message(&mut self, message: Message, _data: Option<Vec<u8>>) {
        match message {
            Message::Send(send) => {
                let _ = self.tx.send(send.payload);
            }
            Message::Log(log) => println!("[{:?}] {}", log.level, log.payload),
            Message::Error(err) => eprintln!(
                "[error] {} at {}:{}:{}",
                err.description, err.file_name, err.line_number, err.column_number
            ),
            Message::Other(val) => println!("[msg] {:?}", val),
        }
    }
}

pub fn load_source(file: Option<&Path>, eval: Option<&str>) -> anyhow::Result<String> {
    if let Some(path) = file {
        return Ok(std::fs::read_to_string(path)?);
    }
    if let Some(code) = eval {
        return Ok(code.to_string());
    }
    Ok(r#"
        var mods = Process.enumerateModules();
        console.log('[lancer-frida] attached — pid ' + Process.id + ' (' + Process.arch + ') ' + mods.length + ' modules');
        mods.slice(0, 20).forEach(function(m) {
            console.log('  ' + m.name + ' @ ' + m.base + '  size=' + m.size);
        });
        if (mods.length > 20) console.log('  ... and ' + (mods.length - 20) + ' more');
    "#
    .to_string())
}
