use std::path::Path;

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
