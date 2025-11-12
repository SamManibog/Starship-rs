use crate::IdManager;

#[derive(Debug)]
pub struct LivePluginIdManager {
    synth: IdManager<LivePluginId>,
    drum: IdManager<LivePluginId>,
    effect: IdManager<LivePluginId>,
}

impl LivePluginIdManager {
    pub fn new() -> Self {
        type Lci = LivePluginId;
        Self {
            synth: IdManager::new(Lci::SYNTH_MIN, Lci::SYNTH_MAX),
            drum: IdManager::new(Lci::DRUM_MIN, Lci::DRUM_MAX),
            effect: IdManager::new(Lci::EFFECT_MIN, Lci::EFFECT_MAX),
        }
    }

    fn manager_mut(&mut self, kind: LivePluginKind) -> &mut IdManager<LivePluginId> {
        match kind {
            LivePluginKind::None => panic!("Attempted to get manager for 'None' component type."),
            LivePluginKind::Synth => &mut self.synth,
            LivePluginKind::Drum => &mut self.drum,
            LivePluginKind::Effect => &mut self.effect,
        }
    }

    fn manager(&self, kind: LivePluginKind) -> &IdManager<LivePluginId> {
        match kind {
            LivePluginKind::None => panic!("Attempted to get manager for 'None' component type."),
            LivePluginKind::Synth => &self.synth,
            LivePluginKind::Drum => &self.drum,
            LivePluginKind::Effect => &self.effect,
        }
    }

    pub fn get_id(&mut self, kind: LivePluginKind) -> Option<LivePluginId> {
        if kind == LivePluginKind::None {
            Some(LivePluginId::NONE)
        } else {
            self.manager_mut(kind).get_id()
        }
    }

    pub fn give_id(&mut self, id: LivePluginId) {
        let kind = id.kind();
        if kind != LivePluginKind::None {
            self.manager_mut(kind).give_id(id);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LivePluginId {
    id: u32
}

impl LivePluginId {
    const SYNTH_MIN: u32 = 1;
    const SYNTH_MAX: u32 = 1 << 31;
    const DRUM_MIN: u32 = (1 << 31) + 1;
    const DRUM_MAX: u32 = 2 << 31;
    const EFFECT_MIN: u32 = (2 << 31) + 1;
    const EFFECT_MAX: u32 = u32::MAX;
    
    pub const NONE: Self = LivePluginId { id: 0 };

    pub fn kind(&self) -> LivePluginKind {
        if self.id >= Self::EFFECT_MIN {
            LivePluginKind::Effect
        } else if self.id >= Self::DRUM_MIN {
            LivePluginKind::Drum
        } else if self.id >= Self::SYNTH_MIN {
            LivePluginKind::Synth
        } else {
            LivePluginKind::None
        }
    }
}

impl Into<u32> for LivePluginId {
    fn into(self) -> u32 {
        self.id
    }
}

impl From<u32> for LivePluginId {
    fn from(value: u32) -> Self {
        Self {id: value}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LivePluginKind {
    None,
    Synth,
    Drum,
    Effect
}

