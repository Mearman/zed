use std::path::PathBuf;

use crate::AgentServerCommand;
use anyhow::Result;
use collections::HashMap;
use gpui::{App, SharedString};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings::{Settings, SettingsSources};

pub fn init(cx: &mut App) {
    AllAgentServersSettings::register(cx);
}

#[derive(Default, Deserialize, Serialize, Clone, JsonSchema, Debug)]
pub struct AllAgentServersSettings {
    pub gemini: Option<BuiltinAgentServerSettings>,
    pub claude: Option<BuiltinAgentServerSettings>,

    /// Custom agent servers configured by the user
    #[serde(flatten)]
    pub custom: HashMap<SharedString, CustomAgentServerSettings>,
}

#[derive(Default, Deserialize, Serialize, Clone, JsonSchema, Debug, PartialEq)]
pub struct BuiltinAgentServerSettings {
    /// Absolute path to a binary to be used when launching this agent.
    ///
    /// This can be used to run a specific binary without automatic downloads or searching `$PATH`.
    #[serde(rename = "command")]
    pub path: Option<PathBuf>,
    /// If a binary is specified in `command`, it will be passed these arguments.
    pub args: Vec<String>,
    /// If a binary is specified in `command`, it will be passed these environment variables.
    pub env: Option<HashMap<String, String>>,
    /// Whether to skip searching `$PATH` for an agent server binary when
    /// launching this agent.
    ///
    /// This has no effect if a `command` is specified. Otherwise, when this is
    /// set to `true`, Zed will automatically install its own private binary and
    /// update it as needed. When it is set to `false`, Zed will search for a
    /// binary on `$PATH` and fail if none is found.
    ///
    /// Default: true
    #[serde(default = "util::serde::default_true")]
    pub ignore_system_version: bool,
}

impl BuiltinAgentServerSettings {
    pub(crate) fn custom_command(self) -> Option<AgentServerCommand> {
        self.path.map(|path| AgentServerCommand {
            path,
            args: self.args,
            env: self.env,
        })
    }
}

impl From<AgentServerCommand> for BuiltinAgentServerSettings {
    fn from(value: AgentServerCommand) -> Self {
        BuiltinAgentServerSettings {
            path: Some(value.path),
            args: value.args,
            env: value.env,
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize, Clone, JsonSchema, Debug, PartialEq)]
pub struct CustomAgentServerSettings {
    #[serde(flatten)]
    pub command: AgentServerCommand,
}

impl settings::Settings for AllAgentServersSettings {
    const KEY: Option<&'static str> = Some("agent_servers");

    type FileContent = Self;

    fn load(sources: SettingsSources<Self::FileContent>, _: &mut App) -> Result<Self> {
        let mut settings = AllAgentServersSettings::default();

        for AllAgentServersSettings {
            gemini,
            claude,
            custom,
        } in sources.defaults_and_customizations()
        {
            if gemini.is_some() {
                settings.gemini = gemini.clone();
            }
            if claude.is_some() {
                settings.claude = claude.clone();
            }

            // Merge custom agents
            for (name, config) in custom {
                // Skip built-in agent names to avoid conflicts
                if name != "gemini" && name != "claude" {
                    settings.custom.insert(name.clone(), config.clone());
                }
            }
        }

        Ok(settings)
    }

    fn import_from_vscode(_vscode: &settings::VsCodeSettings, _current: &mut Self::FileContent) {}
}
