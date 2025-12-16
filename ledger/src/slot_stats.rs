use {
    crate::blockstore_meta::SlotMeta,
    bitflags::bitflags,
    lru::LruCache,
    solana_clock::Slot,
    std::{collections::HashMap, sync::Mutex},
};

const SLOTS_STATS_CACHE_CAPACITY: usize = 300;

#[derive(Copy, Clone, Debug)]
pub(crate) enum ShredSource {
    Turbine,
    Repaired,
    Recovered,
}

bitflags! {
    #[derive(Copy, Clone, Default)]
    struct SlotFlags: u8 {
        const DEAD   = 0b00000001;
        const FULL   = 0b00000010;
        const ROOTED = 0b00000100;
    }
}

#[derive(Clone, Default)]
pub struct SlotStats {
    turbine_fec_set_index_counts: HashMap</*fec_set_index*/ u32, /*count*/ usize>,
    num_repaired: usize,
    num_recovered: usize,
    last_index: u64,
    flags: SlotFlags,
}

impl SlotStats {
    pub fn get_min_index_count(&self) -> usize {
        self.turbine_fec_set_index_counts
            .values()
            .min()
            .copied()
            .unwrap_or_default()
    }

    fn report(&self, slot: Slot) {
        let min_fec_set_count = self.get_min_index_count();
        datapoint_info!(
            "slot_stats_tracking_complete",
            ("slot", slot, i64),
            ("last_index", self.last_index, i64),
            ("num_repaired", self.num_repaired, i64),
            ("num_recovered", self.num_recovered, i64),
            ("min_turbine_fec_set_count", min_fec_set_count, i64),
            ("is_full", self.flags.contains(SlotFlags::FULL), bool),
            ("is_rooted", self.flags.contains(SlotFlags::ROOTED), bool),
            ("is_dead", self.flags.contains(SlotFlags::DEAD), bool),
        );
    }
}

pub struct SlotsStats {
    pub stats: Mutex<LruCache<Slot, SlotStats>>,
}

impl Default for SlotsStats {
    fn default() -> Self {
        Self {
            stats: Mutex::new(LruCache::new(SLOTS_STATS_CACHE_CAPACITY)),
        }
    }
}

impl SlotsStats {
    /// Returns a mutable reference to [`SlotStats`] associated with the slot in the stats LruCache
    /// and a possibly evicted cache entry.
    ///
    /// A new SlotStats entry will be inserted if there is not one present for `slot`; insertion
    /// may cause an existing entry to be evicted.
    fn get_or_default_with_eviction_check(
        stats: &mut LruCache<Slot, SlotStats>,
        slot: Slot,
    ) -> (&mut SlotStats, Option<(Slot, SlotStats)>) {
        let evicted = if stats.contains(&slot) {
            None
        } else {
            // insert slot in cache which might potentially evict an entry
            let evicted = stats.push(slot, SlotStats::default());
            if let Some((evicted_slot, _)) = evicted {
                assert_ne!(evicted_slot, slot);
            }
            evicted
        };
        (stats.get_mut(&slot).unwrap(), evicted)
    }

    pub(crate) fn record_shred(
        &self,
        _slot: Slot,
        _fec_set_index: u32,
        _source: ShredSource,
        _slot_meta: Option<&SlotMeta>,
    ) {
        // DISABLED: Slot stats collection skipped for performance
    }

    fn add_flag(&self, _slot: Slot, _flag: SlotFlags) {
        // DISABLED: Slot stats collection skipped for performance
    }

    pub fn mark_dead(&self, _slot: Slot) {
        // DISABLED: Slot stats collection skipped for performance
    }

    pub fn mark_rooted(&self, _slot: Slot) {
        // DISABLED: Slot stats collection skipped for performance
    }
}
