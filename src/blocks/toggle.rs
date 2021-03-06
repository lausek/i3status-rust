use std::time::Duration;
use std::process::Command;
use chan::Sender;
use scheduler::Task;

use block::{Block, ConfigBlock};
use config::Config;
use de::deserialize_opt_duration;
use errors::*;
use widgets::button::ButtonWidget;
use widget::I3BarWidget;
use input::I3BarEvent;

use uuid::Uuid;

pub struct Toggle {
    text: ButtonWidget,
    command_on: String,
    command_off: String,
    command_state: String,
    update_interval: Option<Duration>,
    toggled: bool,
    id: String,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct ToggleConfig {
    /// Update interval in seconds
    #[serde(default, deserialize_with = "deserialize_opt_duration")]
    pub interval: Option<Duration>,

    /// Shell Command to enable the toggle
    pub command_on: String,

    /// Shell Command to disable the toggle
    pub command_off: String,

    /// Shell Command to determine toggle state. <br/>Empty output => off. Any output => on.
    pub command_state: String,

    /// Text to display in i3bar for this block
    pub text: String,
}

impl Toggle {

    fn execute(&self, cmd: &String) -> bool {
        Command::new("sh")
           .args(&["-c", cmd])
           .output()
           .expect("failed to execute toggle command")
           .status
           .success()
    }
}

impl ConfigBlock for Toggle {
    type Config = ToggleConfig;

    fn new(block_config: Self::Config, config: Config, _tx_update_request: Sender<Task>) -> Result<Self> {
        let id = Uuid::new_v4().simple().to_string();
        Ok(Toggle {
            text: ButtonWidget::new(config, &id).with_text(&block_config.text),
            command_on: block_config.command_on,
            command_off: block_config.command_off,
            command_state: block_config.command_state,
            id,
            toggled: false,
            update_interval: block_config.interval,
        })
    }
}

impl Block for Toggle {

    fn update(&mut self) -> Result<Option<Duration>> {
        self.toggled = self.execute(&self.command_state);
        self.text.set_icon(if self.toggled {
            "toggle_on"
        } else {
            "toggle_off" 
        });

        Ok(self.update_interval)
    }

    fn view(&self) -> Vec<&I3BarWidget> {
        vec![&self.text]
    }

    fn click(&mut self, e: &I3BarEvent) -> Result<()> {
        if let Some(ref name) = e.name {
            if name.as_str() == self.id {

                let cmd = if self.toggled {
                    &self.command_off
                } else {
                    &self.command_on
                };

                if self.execute(&cmd) {
                    self.toggled = !self.toggled;
                    self.text.set_icon(if self.toggled {
                        "toggle_on"
                    } else {
                        "toggle_off" 
                    });
                }
            }
        }

        Ok(())
    }

    fn id(&self) -> &str {
        &self.id
    }
}
