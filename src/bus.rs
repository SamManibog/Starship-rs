use std::collections::HashMap;

use crate::{live_plugin_id::LivePluginId, playback::LiveEffectContainer};

#[derive(Debug)]
pub struct Bus {
    /// the main channels
    channels: [Box<BusChannel>; Self::MAX_CHANNELS],

    /// the master channel
    master_channel: BusChannel,

    /// scratch buffer for inputs from previous pass of effects, not yet processed
    // reasoning behind this is to improve runtime by reducing the amount of memory allocated by
    // calls to update()
    sample_buffer: [f32; Self::MAX_CHANNELS],

}

impl Bus {
    /// the maximum number of channels in a bus
    pub const MAX_CHANNELS: usize = 100;
}

#[derive(Debug)]
pub struct BusChannel {
    /// ids of effects plugins to send to
    effects: [LivePluginId; Self::MAX_EFFECTS],

    /// whether or not the channel is muted
    muted: bool,

    /// the final output volume of the channel as a fraction
    volume: f32,
}

impl BusChannel {
    /// The maximum number of effects in the channel
    pub const MAX_EFFECTS: usize = 8;

    pub fn new() -> Self {
        Self {
            effects: [LivePluginId::NONE; Self::MAX_EFFECTS],
            muted: false,
            volume: 1.0
        }
    }
}

impl Default for BusChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl Bus {
    /// get a sample from the bus
    pub fn update(
        &mut self,
        inputs: &[f32; Self::MAX_CHANNELS],
        effects: &mut HashMap<LivePluginId, LiveEffectContainer>,
        sample_rate: u32
    ) -> f32 {
        // get initial sample
        for i in 0..inputs.len() {
            let effect_id = self.channels[i].effects[0];
            if effect_id != LivePluginId::NONE {
                self.sample_buffer[i] = effects
                    .get_mut(&effect_id)
                    .unwrap()
                    .update(inputs[i], sample_rate);
            }
        }

        // get next samples in channels
        for e in 1..BusChannel::MAX_EFFECTS {
            for i in 0..inputs.len() {
                let effect_id = self.channels[i].effects[e];
                if effect_id != LivePluginId::NONE {
                    self.sample_buffer[i] = effects
                        .get_mut(&effect_id)
                        .unwrap()
                        .update(inputs[i], sample_rate);
                }
            }
        }

        // handle muting/volumes
        let mut master_sample = 0.0;
        for i in 0..inputs.len() {
            let mult = if self.channels[i].muted {
                0.0
            } else {
                self.channels[i].volume
            };
            master_sample += mult * self.sample_buffer[i];
        }

        // compute main channel output
        for e in 0..BusChannel::MAX_EFFECTS {
            let effect_id = self.master_channel.effects[e];
            if effect_id != LivePluginId::NONE {
                master_sample = effects
                    .get_mut(&effect_id)
                    .unwrap()
                    .update(master_sample, sample_rate);
            }
        }

        if self.master_channel.muted {
            0.0
        } else {
            master_sample * self.master_channel.volume
        }
    }
}
