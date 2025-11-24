use crate::IdManager;

#[derive(Debug)]
pub struct LivePluginIdManager {
    synth: IdManager<LivePluginId>,
    drum: IdManager<LivePluginId>,
    effect: IdManager<LivePluginId>,
    effect_group: IdManager<LivePluginId>,
}

impl LivePluginIdManager {
    pub fn new() -> Self {
        type Lci = LivePluginId;
        Self {
            synth: IdManager::new(Lci::SYNTH_MIN, Lci::SYNTH_MAX),
            drum: IdManager::new(Lci::DRUM_MIN, Lci::DRUM_MAX),
            effect: IdManager::new(Lci::EFFECT_MIN, Lci::EFFECT_MAX),
            effect_group: IdManager::new(Lci::EFFECT_GROUP_MIN, Lci::EFFECT_GROUP_MAX),
        }
    }

    fn manager_mut(&mut self, kind: LivePluginKind) -> &mut IdManager<LivePluginId> {
        match kind {
            LivePluginKind::Nil => panic!("Attempted to get manager for 'None' component type."),
            LivePluginKind::Synth => &mut self.synth,
            LivePluginKind::Drum => &mut self.drum,
            LivePluginKind::Effect => &mut self.effect,
            LivePluginKind::EffectGroup => &mut self.effect_group,
        }
    }

    fn manager(&self, kind: LivePluginKind) -> &IdManager<LivePluginId> {
        match kind {
            LivePluginKind::Nil => panic!("Attempted to get manager for 'None' component type."),
            LivePluginKind::Synth => &self.synth,
            LivePluginKind::Drum => &self.drum,
            LivePluginKind::Effect => &self.effect,
            LivePluginKind::EffectGroup => &self.effect_group,
        }
    }

    pub fn get_id(&mut self, kind: LivePluginKind) -> Option<LivePluginId> {
        if kind == LivePluginKind::Nil {
            Some(LivePluginId::NIL)
        } else {
            self.manager_mut(kind).get_id()
        }
    }

    pub fn give_id(&mut self, id: LivePluginId) {
        let kind = id.kind();
        if kind != LivePluginKind::Nil {
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
    const EFFECT_MAX: u32 = 3 << 31;
    const EFFECT_GROUP_MIN: u32 = (3 << 31) + 1;
    const EFFECT_GROUP_MAX: u32 = u32::MAX;
    
    pub const NIL: Self = LivePluginId { id: 0 };

    pub fn kind(&self) -> LivePluginKind {
        if self.id >= Self::EFFECT_GROUP_MIN {
            LivePluginKind::EffectGroup
        } else if self.id >= Self::EFFECT_MIN {
            LivePluginKind::Effect
        } else if self.id >= Self::DRUM_MIN {
            LivePluginKind::Drum
        } else if self.id >= Self::SYNTH_MIN {
            LivePluginKind::Synth
        } else {
            LivePluginKind::Nil
        }
    }

    pub fn is_nil(&self) -> bool {
        *self == Self::NIL
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
    Nil,
    Synth,
    Drum,
    Effect,
    EffectGroup
}

impl LivePluginKind {
    pub fn is_nil(&self) -> bool {
        *self == Self::Nil
    }
}
