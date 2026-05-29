use crate::DeviceArgs;

/// Returns the CLI flags for device selection: ["-U"], ["-H", "host"], or [].
pub fn device_flags(args: &DeviceArgs) -> Vec<String> {
    if args.usb {
        vec!["-U".to_string()]
    } else if let Some(ref host) = args.remote {
        vec!["-H".to_string(), host.clone()]
    } else {
        vec![]
    }
}
