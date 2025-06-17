use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub general: General,
    pub theme: Theme,
    pub bar: Bar,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct General {
    pub time_format: TimeFormat,
    pub unit: Unit,
    /// Get from https://www.latlong.net/
    pub lat: f64,
    /// Get from https://www.latlong.net/
    pub lon: f64,
    pub debug: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeFormat {
    #[serde(rename = "12h")]
    Twelve,
    #[serde(rename = "24h")]
    TwentyFour,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Metric,
    Imperial,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    pub font: String,
    pub border_radius: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bar {
    pub theme: BarTheme,
    pub modules: Modules,
    pub app_launcher: AppLauncher,
    pub battery: Battery,
    pub clock: Clock,
    pub notifications: Notifications,
    pub quick_settings: QuickSettings,
    pub sysmon: SysMon,
    pub todo: Todo,
    pub twitch: Twitch,
    pub workspaces: Workspaces,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BarTheme {
    pub bg: String,
    pub section_bg: String,
    pub fg: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModuleName {
    AppLauncher,
    Battery,
    Clock,
    Music,
    Notifications,
    QuickSettings,
    Sysmon,
    Todo,
    Twitch,
    Workspaces,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Modules {
    pub left: Vec<ModuleName>,
    pub center: Vec<ModuleName>,
    pub right: Vec<ModuleName>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MouseCommands {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_clicked: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_primary_click: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_middle_click: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_secondary_click: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_scroll_up: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_scroll_down: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icon {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Text {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppLauncher {
    pub icon: Icon,
    #[serde(flatten)]
    pub mouse_commands: MouseCommands,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Battery {
    pub ramp_icons: Vec<Icon>,
    pub charging_icon: Icon,
    pub full_threshold: u32,
    #[serde(flatten)]
    pub mouse_commands: MouseCommands,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Clock {
    pub icon: Icon,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationSegment {
    Weather,
    Todoist,
    Github,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notifications {
    pub segments: Vec<NotificationSegment>,
    pub weather: Weather,
    pub todoist: Todoist,
    pub github: Github,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemperatureType {
    Apparent,
    Exact,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Weather {
    pub temperature: TemperatureType,
    pub icon: WeatherIcon,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Text>,
    #[serde(flatten)]
    pub mouse_commands: MouseCommands,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherIcon {
    pub color: WeatherColors,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherColors {
    pub day: String,
    pub night: String,
    pub rain: String,
    pub snow: String,
    pub fog: String,
    pub wind: String,
    pub cloud: String,
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Todoist {
    pub icon: Icon,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Text>,
    /// Get from https://todoist.com/prefs/integrations
    pub api_token: String,
    #[serde(flatten)]
    pub mouse_commands: MouseCommands,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Github {
    pub icon: Icon,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Text>,
    /// Classic token with the 'notifications' scope
    pub api_token: String,
    #[serde(flatten)]
    pub mouse_commands: MouseCommands,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QuickSettingSegment {
    Tray,
    NightLight,
    Brightness,
    Caffeine,
    Dnd,
    Mic,
    Notifs,
    Volume,
    Network,
    Battery,
    Toggle,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickSettings {
    pub segments: Vec<QuickSettingSegment>,
    pub night_light: NightLight,
    pub tray: Tray,
    pub brightness: Brightness,
    pub caffeine: Caffeine,
    pub dnd: Dnd,
    pub mic: Mic,
    pub notifs: Notifs,
    pub volume: Volume,
    pub network: Network,
    pub battery: QuickSettingsBattery,
    pub toggle: Toggle,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NightLight {
    pub off_icon: Icon,
    pub on_icon: Icon,
    pub forced_icon: Icon,
    pub day_temp: u32,
    pub night_temp: u32,
    pub fade_duration_minutes: u32,
    #[serde(flatten)]
    pub mouse_commands: MouseCommands,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tray {
    pub ignored_apps: Vec<String>,
    pub app_icon_mappings: HashMap<String, Icon>,
    #[serde(skip)]
    pub sort_function: Option<Box<dyn Fn(&TrayItem, &TrayItem) -> std::cmp::Ordering>>,
}

// Helper struct for sort function
#[derive(Debug)]
pub struct TrayItem {
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brightness {
    pub ramp_icons: Vec<Icon>,
    pub step: f32,
    #[serde(flatten)]
    pub mouse_commands: MouseCommands,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Caffeine {
    pub icon: Icon,
    pub active_icon: Icon,
    pub toggle_cmd: String,
    #[serde(flatten)]
    pub mouse_commands: MouseCommands,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dnd {
    pub icon: Icon,
    pub dnd_icon: Icon,
    pub toggle_cmd: String,
    pub status_cmd: String,
    pub history_cmd: String,
    pub dismiss_cmd: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mic {
    pub icon: Icon,
    pub mute_icon: Icon,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_secondary_click: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notifs {
    pub icon: Icon,
    pub notifs_icon: Icon,
    pub open_cmd: String,
    pub status_cmd: String,
    pub toggle_cmd: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub ramp_icons: Vec<Icon>,
    pub mute_icon: Icon,
    pub headphones_mute: Icon,
    pub headphones_ramp: Vec<Icon>,
    pub step: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_secondary_click: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Network {
    pub ramp_icons: Vec<Icon>,
    pub off_icon: Icon,
    pub ethernet_icon: Icon,
    pub ethernet_off_icon: Icon,
    #[serde(flatten)]
    pub mouse_commands: MouseCommands,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickSettingsBattery {
    pub ramp_icons: Vec<Icon>,
    pub charging_icon: Icon,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Toggle {
    pub icon: Icon,
    pub open_icon: Icon,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SysMonSegment {
    Cpu,
    Temp,
    Ram,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysMon {
    pub segments: Vec<SysMonSegment>,
    pub probe_interval_ms: u32,
    pub precision: u32,
    pub cpu: Cpu,
    pub temp: Temp,
    pub ram: Ram,
    #[serde(flatten)]
    pub mouse_commands: MouseCommands,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cpu {
    pub icon: Icon,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Text>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Temp {
    pub icon: Icon,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Text>,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ram {
    pub icon: Icon,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Text>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub sound_url: String,
    pub icon: Icon,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Twitch {
    pub icon: Icon,
    pub channels: Vec<String>,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Workspaces {
    pub monitors: Vec<Monitor>,
    pub color: WorkspaceColors,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Monitor {
    pub workspaces: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceColors {
    pub active: String,
    pub has_windows: String,
    pub normal: String,
}
