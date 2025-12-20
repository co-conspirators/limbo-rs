mod battery;
mod clock;
mod quick_settings;
mod sysmon;
mod workspaces;

pub use battery::BatteryView;
pub use clock::Clock;
pub use quick_settings::QuickSettings;
pub use sysmon::{SysInfo, Sysmon};
pub use workspaces::Workspaces;
