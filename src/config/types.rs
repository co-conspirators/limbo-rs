use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub general: General,
    pub theme: Theme,
    pub bar: Bar,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct General {
    pub time_format: TimeFormat,
    pub unit: Unit,
    /// Get from <https://www.latlong.net/>
    pub lat: f64,
    /// Get from <https://www.latlong.net/>
    pub lon: f64,
    pub debug: bool,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeFormat {
    #[default]
    #[serde(rename = "12h")]
    _12h,
    #[serde(rename = "24h")]
    _24h,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    #[default]
    Metric,
    Imperial,
}

/// Wrapper for [`iced::Color`] implementing Serialize and Deserialize
#[derive(Debug, Clone, Copy)]
pub struct Color(pub iced::Color);

impl Color {
    pub fn parse(s: &str) -> Option<Self> {
        iced::Color::parse(s).map(Self)
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let rgba = self.0.into_rgba8();
        let s = format!("#{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2]); // ignore alpha
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ColorVisitor;
        impl<'de> serde::de::Visitor<'de> for ColorVisitor {
            type Value = Color;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a color string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Color::parse(v).ok_or(serde::de::Error::custom("invalid color string"))
            }
        }
        deserializer.deserialize_str(ColorVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ColorNameOrHex {
    Name(String),
    Hex(Color),
}

impl ColorNameOrHex {
    pub fn name(name: impl Into<String>) -> Self {
        ColorNameOrHex::Name(name.into())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    pub font: String,
    pub border_radius: f32,
    pub colors: HashMap<String, Color>,
}

impl Theme {
    pub fn resolve_color(&self, color: &ColorNameOrHex) -> Option<iced::Color> {
        match color {
            ColorNameOrHex::Name(name) => self.colors.get(name).map(|c| c.0),
            ColorNameOrHex::Hex(color) => Some(color.0),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            font: "IBM Plex Mono".to_string(),
            border_radius: 6.0,
            colors: HashMap::from(
                [
                    ("crust", "#11111b"),
                    ("mantle", "#181825"),
                    ("base", "#1e1e2e"),
                    ("core", "#2c2c3f"),
                    ("surface0", "#313244"),
                    ("surface1", "#45475a"),
                    ("surface2", "#585b70"),
                    ("overlay0", "#6c7086"),
                    ("overlay1", "#7f849c"),
                    ("overlay2", "#9399b2"),
                    ("subtext0", "#a6adc8"),
                    ("subtext1", "#bac2de"),
                    ("subtext2", "#cdd6f4"),
                    ("text", "#f0f4ff"),
                    ("lavender", "#b4befe"),
                    ("lavenderDark", "#7f8cfe"),
                    ("blue", "#89b4fa"),
                    ("blueDark", "#5f8cfb"),
                    ("sapphire", "#74c7ec"),
                    ("sapphireDark", "#4a9edc"),
                    ("sky", "#89dceb"),
                    ("skyDark", "#5f9edc"),
                    ("teal", "#94e2d5"),
                    ("tealDark", "#5fb9a8"),
                    ("green", "#a6e3a1"),
                    ("greenDark", "#5fbf6b"),
                    ("yellow", "#f9e2af"),
                    ("yellowDark", "#f5c77b"),
                    ("peach", "#fab387"),
                    ("peachDark", "#f5a87b"),
                    ("maroon", "#eba0ac"),
                    ("maroonDark", "#c97b84"),
                    ("red", "#f38ba8"),
                    ("redDark", "#c97b84"),
                    ("mauve", "#cba6f7"),
                    ("mauveDark", "#a17be3"),
                    ("pink", "#f5c2e7"),
                    ("flamingo", "#f2cdcd"),
                    ("rosewater", "#f5e0dc"),
                    ("cyan", "#bee4ed"),
                ]
                .map(|(name, hex)| (name.to_string(), Color::parse(hex).unwrap())),
            ),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bar {
    pub theme: BarTheme,
    pub modules: Modules,
    pub app_launcher: AppLauncher,
    pub battery: Battery,
    pub clock: Clock,
    pub notifications: Notifications,
    pub quick_settings: QuickSettings,
    pub sysmon: Sysmon,
    pub todo: Todo,
    pub workspaces: Workspaces,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BarTheme {
    pub bg: ColorNameOrHex,
    pub section_bg: ColorNameOrHex,
    pub fg: ColorNameOrHex,
}

impl Default for BarTheme {
    fn default() -> Self {
        Self {
            bg: ColorNameOrHex::name("base"),
            section_bg: ColorNameOrHex::name("core"),
            fg: ColorNameOrHex::name("text"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
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
    Workspaces,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Modules {
    pub left: Vec<ModuleName>,
    pub center: Vec<ModuleName>,
    pub right: Vec<ModuleName>,
}

impl Default for Modules {
    fn default() -> Self {
        Self {
            left: vec![
                ModuleName::AppLauncher,
                ModuleName::Notifications,
                ModuleName::Music,
            ],
            center: vec![ModuleName::Workspaces],
            right: vec![
                ModuleName::Sysmon,
                ModuleName::QuickSettings,
                ModuleName::Battery,
                ModuleName::Clock,
            ],
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
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
    pub color: ColorNameOrHex,
}

impl Icon {
    pub fn new(name: impl Into<String>, color: impl Into<ColorNameOrHex>) -> Self {
        Self {
            name: name.into(),
            color: color.into(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Text {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorNameOrHex>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppLauncher {
    pub icon: Icon,
    #[serde(flatten)]
    pub mouse_commands: MouseCommands,
}

impl Default for AppLauncher {
    fn default() -> Self {
        Self {
            icon: Icon::new("nix-snowflake-white", ColorNameOrHex::name("text")),
            mouse_commands: MouseCommands {
                on_primary_click: Some("tofi-drun".to_string()),
                ..Default::default()
            },
        }
    }
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

impl Default for Battery {
    fn default() -> Self {
        Self {
            ramp_icons: vec![
                Icon::new("battery-1", ColorNameOrHex::name("red")),
                Icon::new("battery-2", ColorNameOrHex::name("yellow")),
                Icon::new("battery-3", ColorNameOrHex::name("green")),
                Icon::new("battery-4", ColorNameOrHex::name("green")),
            ],
            charging_icon: Icon::new("battery-charging", ColorNameOrHex::name("green")),
            full_threshold: 97,
            mouse_commands: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Clock {
    pub icon: Icon,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            icon: Icon::new("clock", ColorNameOrHex::name("text")),
        }
    }
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

impl Default for Notifications {
    fn default() -> Self {
        Self {
            segments: vec![
                NotificationSegment::Weather,
                NotificationSegment::Todoist,
                NotificationSegment::Github,
            ],
            weather: Default::default(),
            todoist: Default::default(),
            github: Default::default(),
        }
    }
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

impl Default for Weather {
    fn default() -> Self {
        Self {
            temperature: TemperatureType::Apparent,
            icon: Default::default(),
            text: None,
            mouse_commands: MouseCommands {
                on_primary_click: Some("xdg-open https://merrysky.net".to_string()),
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WeatherIcon {
    pub color: WeatherColors,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherColors {
    pub day: ColorNameOrHex,
    pub night: ColorNameOrHex,
    pub rain: ColorNameOrHex,
    pub snow: ColorNameOrHex,
    pub fog: ColorNameOrHex,
    pub wind: ColorNameOrHex,
    pub cloud: ColorNameOrHex,
    pub error: ColorNameOrHex,
}

impl Default for WeatherColors {
    fn default() -> Self {
        Self {
            day: ColorNameOrHex::name("yellow"),
            night: ColorNameOrHex::name("blue"),
            rain: ColorNameOrHex::name("blue"),
            snow: ColorNameOrHex::name("text"),
            fog: ColorNameOrHex::name("text"),
            wind: ColorNameOrHex::name("text"),
            cloud: ColorNameOrHex::name("text"),
            error: ColorNameOrHex::name("red"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Todoist {
    pub icon: Icon,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Text>,
    /// Get from <https://todoist.com/prefs/integrations>
    pub api_token: String,
    #[serde(flatten)]
    pub mouse_commands: MouseCommands,
}

impl Default for Todoist {
    fn default() -> Self {
        Self {
            icon: Icon::new("checkbox", ColorNameOrHex::name("red")),
            text: None,
            api_token: String::new(),
            mouse_commands: Default::default(),
        }
    }
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

impl Default for Github {
    fn default() -> Self {
        Self {
            icon: Icon::new("brand-github", ColorNameOrHex::name("text")),
            text: None,
            api_token: String::new(),
            mouse_commands: MouseCommands {
                on_primary_click: Some("xdg-open https://github.com/notifications".to_string()),
                ..Default::default()
            },
        }
    }
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
    pub tray: Tray,
    pub night_light: NightLight,
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

impl Default for QuickSettings {
    fn default() -> Self {
        Self {
            segments: vec![
                QuickSettingSegment::Tray,
                QuickSettingSegment::NightLight,
                QuickSettingSegment::Brightness,
                QuickSettingSegment::Caffeine,
                QuickSettingSegment::Dnd,
                QuickSettingSegment::Mic,
                QuickSettingSegment::Volume,
                QuickSettingSegment::Network,
                QuickSettingSegment::Toggle,
            ],
            tray: Default::default(),
            night_light: Default::default(),
            brightness: Default::default(),
            caffeine: Default::default(),
            dnd: Default::default(),
            mic: Default::default(),
            notifs: Default::default(),
            volume: Default::default(),
            network: Default::default(),
            battery: Default::default(),
            toggle: Default::default(),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tray {
    pub ignored_apps: Vec<String>,
    pub app_icon_mappings: HashMap<String, Icon>,
    #[serde(skip)]
    pub sort_function:
        Option<Box<dyn Fn(&crate::tray::TrayItem, &crate::tray::TrayItem) -> std::cmp::Ordering>>,
}

impl core::fmt::Debug for Tray {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let Tray {
            ignored_apps,
            app_icon_mappings,
            ..
        } = self;
        f.debug_struct("Tray")
            .field("ignored_apps", &ignored_apps)
            .field("app_icon_mappings", &app_icon_mappings)
            .finish()
    }
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

impl Default for NightLight {
    fn default() -> Self {
        Self {
            off_icon: Icon::new("moon-off", ColorNameOrHex::name("yellow")),
            on_icon: Icon::new("moon", ColorNameOrHex::name("yellow")),
            forced_icon: Icon::new("moon-start", ColorNameOrHex::name("yellow")),
            day_temp: 6500,
            night_temp: 4000,
            fade_duration_minutes: 30,
            mouse_commands: Default::default(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brightness {
    pub ramp_icons: Vec<Icon>,
    pub step: f32,
    #[serde(flatten)]
    pub mouse_commands: MouseCommands,
}

impl Default for Brightness {
    fn default() -> Self {
        Self {
            ramp_icons: vec![
                Icon::new("brightness-down", ColorNameOrHex::name("yellow")),
                Icon::new("brightness-half", ColorNameOrHex::name("yellow")),
                Icon::new("brightness-up", ColorNameOrHex::name("yellow")),
            ],
            step: 0.05,
            mouse_commands: Default::default(),
        }
    }
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

impl Default for Caffeine {
    fn default() -> Self {
        Self {
            icon: Icon::new("mug-off", ColorNameOrHex::name("blue")),
            active_icon: Icon::new("coffee", ColorNameOrHex::name("cyan")),
            toggle_cmd: "wlinhibit".to_string(),
            mouse_commands: Default::default(),
        }
    }
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

impl Default for Dnd {
    fn default() -> Self {
        Self {
            icon: Icon::new("bell", ColorNameOrHex::name("red")),
            dnd_icon: Icon::new("bell-off", ColorNameOrHex::name("red")),
            toggle_cmd: "makoctl mode -t do-not-disturb".to_string(),
            status_cmd: "makoctl mode".to_string(),
            history_cmd: "makoctl restore".to_string(),
            dismiss_cmd: "makoctl dismiss".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mic {
    pub icon: Icon,
    pub mute_icon: Icon,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_secondary_click: Option<String>,
}

impl Default for Mic {
    fn default() -> Self {
        Self {
            icon: Icon::new("microphone", ColorNameOrHex::name("pink")),
            mute_icon: Icon::new("microphone-off", ColorNameOrHex::name("red")),
            on_secondary_click: Some("pavucontrol --tab=4".to_string()),
        }
    }
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

impl Default for Notifs {
    fn default() -> Self {
        Self {
            icon: Icon::new("bell", ColorNameOrHex::name("red")),
            notifs_icon: Icon::new("bell-off", ColorNameOrHex::name("red")),
            open_cmd: "swaync-client -t -sw".to_string(),
            status_cmd: "swaync-client -D".to_string(),
            toggle_cmd: "swaync-client -d".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub ramp_icons: Vec<Icon>,
    pub mute_icon: Icon,
    pub headphones_ramp: Vec<Icon>,
    pub headphones_mute: Icon,
    pub step: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_secondary_click: Option<String>,
}

impl Default for Volume {
    fn default() -> Self {
        Self {
            ramp_icons: vec![
                Icon::new("volume-3", ColorNameOrHex::name("flamingo")),
                Icon::new("volume-2", ColorNameOrHex::name("flamingo")),
                Icon::new("volume", ColorNameOrHex::name("flamingo")),
            ],
            mute_icon: Icon::new("volume-off", ColorNameOrHex::name("red")),
            headphones_ramp: vec![
                Icon::new("headphones-off", ColorNameOrHex::name("flamingo")),
                Icon::new("headphones", ColorNameOrHex::name("flamingo")),
            ],
            headphones_mute: Icon::new("volume-off", ColorNameOrHex::name("red")),
            step: 0.05,
            on_secondary_click: Some("pavucontrol --tab=3".to_string()),
        }
    }
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

impl Default for Network {
    fn default() -> Self {
        Self {
            ramp_icons: vec![
                Icon::new("wifi", ColorNameOrHex::name("blue")),
                Icon::new("wifi-2", ColorNameOrHex::name("blue")),
                Icon::new("wifi-1", ColorNameOrHex::name("blue")),
            ],
            off_icon: Icon::new("wifi-off", ColorNameOrHex::name("red")),
            ethernet_icon: Icon::new("ethernet", ColorNameOrHex::name("sky")),
            ethernet_off_icon: Icon::new("ethernet-off", ColorNameOrHex::name("red")),
            mouse_commands: MouseCommands {
                on_primary_click: Some("nm-connection-editor".to_string()),
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickSettingsBattery {
    pub ramp_icons: Vec<Icon>,
    pub charging_icon: Icon,
}

impl Default for QuickSettingsBattery {
    fn default() -> Self {
        Self {
            ramp_icons: vec![
                Icon::new("battery-4", ColorNameOrHex::name("green")),
                Icon::new("battery-3", ColorNameOrHex::name("green")),
                Icon::new("battery-2", ColorNameOrHex::name("green")),
                Icon::new("battery-1", ColorNameOrHex::name("yellow")),
                Icon::new("battery", ColorNameOrHex::name("red")),
            ],
            charging_icon: Icon::new("battery-charging", ColorNameOrHex::name("yellow")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Toggle {
    pub icon: Icon,
    pub open_icon: Icon,
}

impl Default for Toggle {
    fn default() -> Self {
        Self {
            icon: Icon::new("chevron-down", ColorNameOrHex::name("text")),
            open_icon: Icon::new("chevron-up", ColorNameOrHex::name("text")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SysmonSegment {
    Cpu,
    Temp,
    Ram,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sysmon {
    pub segments: Vec<SysmonSegment>,
    pub probe_interval_ms: u64,
    pub cpu: Cpu,
    pub temp: Temp,
    pub ram: Ram,
    #[serde(flatten)]
    pub mouse_commands: MouseCommands,
}

impl Default for Sysmon {
    fn default() -> Self {
        Self {
            segments: vec![SysmonSegment::Cpu, SysmonSegment::Temp, SysmonSegment::Ram],
            probe_interval_ms: 5000,
            cpu: Default::default(),
            temp: Default::default(),
            ram: Default::default(),
            mouse_commands: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cpu {
    pub icon: Icon,
    pub precision: usize,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            icon: Icon::new("cpu", ColorNameOrHex::name("lavender")),
            precision: 1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Temp {
    pub icon: Icon,
    pub precision: usize,
}

impl Default for Temp {
    fn default() -> Self {
        Self {
            icon: Icon::new("temperature", ColorNameOrHex::name("red")),
            precision: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ram {
    pub icon: Icon,
    pub precision: usize,
}

impl Default for Ram {
    fn default() -> Self {
        Self {
            icon: Icon::new("cpu-2", ColorNameOrHex::name("pink")),
            precision: 1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub sound_url: String,
    pub icon: Icon,
}

impl Default for Todo {
    fn default() -> Self {
        Self {
            sound_url:
                "https://todoist.b-cdn.net/assets/sounds/d8040624c9c7c88aa730f73faa60cf39.mp3"
                    .to_string(),
            icon: Icon::new("square", ColorNameOrHex::name("red")),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Workspaces {
    pub color: WorkspaceColors,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceColors {
    pub active: ColorNameOrHex,
    pub has_windows: ColorNameOrHex,
    pub normal: ColorNameOrHex,
}

impl Default for WorkspaceColors {
    fn default() -> Self {
        Self {
            active: ColorNameOrHex::name("blue"),
            has_windows: ColorNameOrHex::name("blue"),
            normal: ColorNameOrHex::name("surface2"),
        }
    }
}
