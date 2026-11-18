//! State for the block detail view (single block header + transaction list).

use starknet::core::types::Felt;

use crate::data::types::{SnBlock, SnTransaction};
use crate::decode::outside_execution::OutsideExecutionVersion;
use crate::ui::widgets::stateful_list::StatefulList;

/// Lightweight meta tx summary for the block view (no inner calls, just intender + version).
#[derive(Debug, Clone)]
pub struct MetaTxSummary {
    pub intender: Felt,
    pub version: OutsideExecutionVersion,
}

/// All state related to the block detail view.
#[derive(Default)]
pub struct BlockDetailState {
    pub block: Option<SnBlock>,
    pub txs: StatefulList<SnTransaction>,
    /// Decoded endpoint/function name per tx (by index in txs).
    pub endpoint_names: Vec<Option<String>>,
    /// Execution status per tx: "OK", "REV", "?"
    pub tx_statuses: Vec<String>,
    /// Outside execution summary per tx. Some for meta txs, None otherwise.
    pub meta_tx_info: Vec<Option<MetaTxSummary>>,
    /// True for any tx that touches the privacy pool (top-level call or
    /// OE-inner call). Drives the orange shield marker in the "Prv"
    /// column on the block tx list.
    pub is_privacy_tx: Vec<bool>,
    /// Whether visual mode (sender selection) is active.
    pub visual_mode: bool,
    /// Cursor index into txs (only meaningful when visual_mode is true).
    pub nav_cursor: usize,
    /// Estimated sequencer batch number per tx (by index in txs); empty when unknown.
    pub batches: Vec<u32>,
}

impl BlockDetailState {
    /// Clear all block detail data. Called when navigating to a new block.
    pub fn clear(&mut self) {
        self.block = None;
        self.txs = StatefulList::new();
        self.endpoint_names = Vec::new();
        self.tx_statuses = Vec::new();
        self.meta_tx_info = Vec::new();
        self.is_privacy_tx = Vec::new();
        self.visual_mode = false;
        self.nav_cursor = 0;
        self.batches = Vec::new();
    }
}

/// Guess each tx's 200ms batch (1-based, in tx-index order). Within a batch the
/// sequencer orders by descending tip, so a tip increase marks a new batch.
pub fn estimate_batches(starknet_version: &str, txs: &[SnTransaction]) -> Vec<u32> {
    if !has_tip_ordering(starknet_version) {
        return Vec::new();
    }
    batches_from_tips(txs.iter().map(|tx| match tx {
        SnTransaction::L1Handler(_) | SnTransaction::Deploy(_) => None,
        _ => Some(tx.tip()),
    }))
}

/// Tip-ordered batches arrived with 0.14; unparsable versions are assumed recent.
fn has_tip_ordering(version: &str) -> bool {
    let mut parts = version.split('.').map(|p| p.parse::<u32>().ok());
    match (parts.next().flatten(), parts.next().flatten()) {
        (Some(major), Some(minor)) => (major, minor) >= (0, 14),
        _ => true,
    }
}

/// Untipped txs (`None`) stay in the current batch and don't reset the comparison.
fn batches_from_tips(tips: impl Iterator<Item = Option<u64>>) -> Vec<u32> {
    let mut batch = 1;
    let mut prev_tip = None;
    tips.map(|tip| {
        if let Some(tip) = tip {
            if prev_tip.is_some_and(|prev| tip > prev) {
                batch += 1;
            }
            prev_tip = Some(tip);
        }
        batch
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tip_increase_starts_new_batch() {
        // Mainnet block 18142610.
        let tips: [u64; 42] = [
            1,
            93_271_235_770_031,
            44_392_478_769_831,
            58_635_532_980,
            51_779_459_453_348,
            27_825_821_868_253,
            25_557_879_078_890,
            16_439_979_069_218,
            9_877_508_151_470,
            4_073_394_475_465,
            2_797_125_090_844,
            421_930_449_003,
            369_236_738_970,
            218_298_861_417,
            1_532_419_865_312,
            741_665_259_727,
            541_180_798_982,
            485_979_602_928,
            428_734_561_060,
            318_453_043_752,
            210_928_004_813,
            210_580_087_105,
            2_323_818_853_586,
            850_000_000_000,
            725_224_100_437,
            318_341_147_611,
            221_570_940_557,
            204_078_849_669,
            172_841_575_675,
            165_955_425_569,
            162_576_495_157,
            147_134_035_401,
            142_490_628_923,
            139_489_190_186,
            80_848_269_797,
            76_975_917_305,
            75_269_193_381,
            72_500_000_001,
            63_541_296_020,
            52_783_850_566,
            44_858_106_751,
            41_661_313_914,
        ];
        let batches = batches_from_tips(tips.into_iter().map(Some));
        let sizes: Vec<usize> = batches.chunk_by(|a, b| a == b).map(<[u32]>::len).collect();
        assert_eq!(sizes, [1, 3, 10, 8, 20]);
        assert_eq!(batches.last(), Some(&5));
    }

    #[test]
    fn untipped_txs_do_not_split_batches() {
        let tips = [Some(50), None, Some(40), None, Some(60)];
        assert_eq!(batches_from_tips(tips.into_iter()), [1, 1, 1, 1, 2]);
    }

    #[test]
    fn pre_tip_versions_are_skipped() {
        assert!(!has_tip_ordering("0.13.4"));
        assert!(has_tip_ordering("0.14.0"));
        assert!(has_tip_ordering("0.14.1"));
        assert!(has_tip_ordering("1.0"));
        assert!(has_tip_ordering(""));
    }
}
