use bfieldcodec_derive::BFieldCodec;
use itertools::Itertools;

use crate::{
    prelude::{AlgebraicHasher, Digest, Mmr, Tip5},
    util_types::mmr::shared_advanced::{left_sibling, node_indices_added_by_append},
};

use super::{
    mmr_accumulator::MmrAccumulator,
    shared_advanced::{
        get_peak_heights, get_peak_heights_and_peak_node_indices, parent, right_sibling,
    },
    shared_basic::{calculate_new_peaks_from_append, leaf_index_to_mt_index_and_peak_index},
};

/// An MmrSuccessorProof asserts that one MMR Accumulator is the descendant of
/// another, *i.e.*, that the second can be obtained by appending a set of leafs
/// to the first. It consists of a set of authentication paths connecting the
/// old peaks to the new peaks.
#[derive(Debug, Clone, BFieldCodec)]
pub struct MmrSuccessorProof {
    pub paths: Vec<Vec<Digest>>,
}

impl MmrSuccessorProof {
    /// Compute a new `MmrSuccessorProof` given the starting MMR accumulator and
    /// a list of digests to be appended.
    pub fn new_from_batch_append(mmra: &MmrAccumulator, new_leafs: &[Digest]) -> Self {
        let (heights_of_old_peaks, indices_of_old_peaks) =
            get_peak_heights_and_peak_node_indices(mmra.num_leafs());
        let (_heights_of_new_peaks, indices_of_new_peaks) =
            get_peak_heights_and_peak_node_indices(mmra.num_leafs() + new_leafs.len() as u64);
        let num_old_peaks = heights_of_old_peaks.len();

        let mut needed_indices = vec![vec![]; num_old_peaks];
        for (i, (index, height)) in indices_of_old_peaks
            .iter()
            .copied()
            .zip(heights_of_old_peaks)
            .enumerate()
        {
            let mut current_index = index;
            let mut current_height = height;
            while !indices_of_new_peaks.contains(&current_index) {
                let mut sibling = right_sibling(current_index, current_height);
                let parent_index = parent(current_index);
                if parent(sibling) != parent_index {
                    sibling = left_sibling(current_index, current_height);
                };
                let list_index = needed_indices[i].len();
                needed_indices[i].push(Some((list_index, sibling)));
                current_height += 1;
                current_index = parent_index;
            }
        }

        let mut current_peaks = mmra.peaks();
        let mut current_peak_indices = indices_of_old_peaks.clone();
        let mut current_leaf_count = mmra.num_leafs();
        let mut paths = needed_indices
            .iter()
            .map(|ni| vec![Digest::default(); ni.len()])
            .collect_vec();

        for &new_leaf in new_leafs {
            let new_node_indices = node_indices_added_by_append(current_leaf_count);

            let (new_peaks, membership_proof) = calculate_new_peaks_from_append(
                current_leaf_count,
                current_peaks.clone(),
                new_leaf,
            );

            let (_new_heights, new_peak_indices) =
                get_peak_heights_and_peak_node_indices(current_leaf_count + 1);
            let new_nodes = membership_proof
                .authentication_path
                .into_iter()
                .scan(new_leaf, |runner, path_node| {
                    let yld = *runner;
                    *runner = Tip5::hash_pair(path_node, *runner);
                    Some(yld)
                })
                .collect_vec();

            for (index, node) in new_node_indices.into_iter().zip(new_nodes).chain(
                current_peak_indices
                    .into_iter()
                    .zip(current_peaks.iter().copied()),
            ) {
                for (path, path_indices) in paths.iter_mut().zip(needed_indices.iter_mut()) {
                    if let Some(wrapped_pair) = path_indices
                        .iter_mut()
                        .filter(|maybe| maybe.is_some())
                        .find(|definitely| definitely.unwrap().1 == index)
                    {
                        path[wrapped_pair.unwrap().0] = node;
                        *wrapped_pair = None;
                    }
                }
            }

            current_peaks = new_peaks;
            current_peak_indices = new_peak_indices;
            current_leaf_count += 1;
        }

        Self { paths }
    }

    /// Verify that `old_mmra` is a predecessor of `new_mmra`.
    pub fn verify(&self, old_mmra: &MmrAccumulator, new_mmra: &MmrAccumulator) -> bool {
        if old_mmra.num_leafs() == 0 {
            return true;
        }

        let old_peak_heights = get_peak_heights(old_mmra.num_leafs());
        if old_peak_heights.len() != self.paths.len() {
            return false;
        }

        let new_peak_heights = get_peak_heights(new_mmra.num_leafs());

        let mut running_leaf_count = 0;
        for (starting_peak_idx, (old_peak, old_height)) in old_mmra
            .peaks()
            .into_iter()
            .zip(old_peak_heights.into_iter())
            .enumerate()
        {
            running_leaf_count += 1 << old_height;
            if running_leaf_count > new_mmra.num_leafs() {
                return false;
            }

            let mut current_height = old_height;
            let mut current_node = old_peak;
            let (merkle_tree_index_of_last_leaf_under_this_peak, _) =
                leaf_index_to_mt_index_and_peak_index(running_leaf_count - 1, new_mmra.num_leafs());
            let mut current_merkle_tree_index =
                merkle_tree_index_of_last_leaf_under_this_peak >> current_height;

            for &sibling in self.paths[starting_peak_idx].iter() {
                let is_left_sibling = current_merkle_tree_index & 1 == 0;
                current_node = if is_left_sibling {
                    Tip5::hash_pair(current_node, sibling)
                } else {
                    Tip5::hash_pair(sibling, current_node)
                };
                current_merkle_tree_index >>= 1;
                current_height += 1;
            }
            if !new_mmra
                .peaks()
                .into_iter()
                .zip(new_peak_heights.iter())
                .enumerate()
                .any(|(landing_peak_idx, (p, h))| {
                    p == current_node
                        && *h == current_height
                        && landing_peak_idx <= starting_peak_idx
                })
            {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use proptest::collection::vec;
    use proptest::prop_assert;
    use proptest_arbitrary_interop::arb;
    use rand::rngs::StdRng;
    use rand::thread_rng;
    use rand::Rng;
    use rand::RngCore;
    use rand::SeedableRng;
    use test_strategy::proptest;

    use super::MmrSuccessorProof;
    use crate::prelude::Digest;
    use crate::prelude::Mmr;
    use crate::util_types::mmr::mmr_accumulator::MmrAccumulator;

    fn verification_succeeds_with_n_leafs_append_m(n: usize, m: usize, rng: &mut dyn RngCore) {
        let original_leafs = (0..n).map(|_| rng.gen::<Digest>()).collect_vec();
        let old_mmra = MmrAccumulator::new_from_leafs(original_leafs);

        let new_leafs = (0..m).map(|_| rng.gen::<Digest>()).collect_vec();
        let successor_proof = MmrSuccessorProof::new_from_batch_append(&old_mmra, &new_leafs);

        let mut new_mmra = old_mmra.clone();
        for new_leaf in new_leafs {
            new_mmra.append(new_leaf);
        }

        assert!(successor_proof.verify(&old_mmra, &new_mmra));
    }

    #[test]
    fn small_leaf_counts_unit() {
        let mut rng = thread_rng();
        let threshold = 7;
        for n in 0..threshold {
            for m in 0..threshold {
                verification_succeeds_with_n_leafs_append_m(n, m, &mut rng);
            }
        }
    }

    #[proptest]
    fn verification_succeeds_positive_property(
        #[strategy(arb::<MmrAccumulator>())] old_mmr: MmrAccumulator,
        #[strategy(vec(arb::<Digest>(), 0usize..(1<<10)))] new_leafs: Vec<Digest>,
    ) {
        let mut new_mmr = old_mmr.clone();
        let mmr_successor_proof = MmrSuccessorProof::new_from_batch_append(&old_mmr, &new_leafs);
        for leaf in new_leafs {
            new_mmr.append(leaf);
        }

        prop_assert!(mmr_successor_proof.verify(&old_mmr, &new_mmr));
    }

    fn rotr(i: u64) -> u64 {
        (i >> 1) | ((i & 1) << 63)
    }

    #[proptest]
    fn verification_fails_negative_properties(
        #[filter(#old_mmr.num_leafs() != rotr(#old_mmr.num_leafs()))]
        #[strategy(arb::<MmrAccumulator>())]
        old_mmr: MmrAccumulator,
        #[strategy(vec(arb::<Digest>(), 0usize..(1<<10)))] new_leafs: Vec<Digest>,
        #[strategy(arb::<usize>())] mut modify_path_element: usize,
    ) {
        let mut new_mmr = old_mmr.clone();
        let mmr_successor_proof = MmrSuccessorProof::new_from_batch_append(&old_mmr, &new_leafs);
        for leaf in new_leafs.iter() {
            new_mmr.append(*leaf);
        }

        // old MMR has wrong num leafs
        if rotr(old_mmr.num_leafs()) != old_mmr.num_leafs()
            && rotr(old_mmr.num_leafs()) < (u64::MAX >> 1)
        {
            let fake_old_mmr = MmrAccumulator::init(old_mmr.peaks(), rotr(old_mmr.num_leafs()));
            prop_assert!(!mmr_successor_proof.verify(&fake_old_mmr, &new_mmr));
        }

        // new MMR has wrong num leafs
        if rotr(new_mmr.num_leafs()) != new_mmr.num_leafs()
            && rotr(new_mmr.num_leafs()) < (u64::MAX >> 1)
        {
            let fake_new_mmr = MmrAccumulator::init(new_mmr.peaks(), rotr(new_mmr.num_leafs()));
            prop_assert!(!mmr_successor_proof.verify(&old_mmr, &fake_new_mmr));
        }

        // change one path element
        if mmr_successor_proof.paths.len() != 0 {
            let mut fake_mmr_successor_proof_3 = mmr_successor_proof.clone();
            let path_index = modify_path_element % fake_mmr_successor_proof_3.paths.len();
            modify_path_element =
                (modify_path_element - path_index) / fake_mmr_successor_proof_3.paths.len();
            if fake_mmr_successor_proof_3.paths[path_index].len() != 0 {
                let node_index_along_path =
                    modify_path_element % fake_mmr_successor_proof_3.paths[path_index].len();
                modify_path_element = (modify_path_element - node_index_along_path)
                    / fake_mmr_successor_proof_3.paths[path_index].len();
                let value_index = modify_path_element % Digest::LEN;
                fake_mmr_successor_proof_3.paths[path_index][node_index_along_path].0[value_index]
                    .increment();
                prop_assert!(!fake_mmr_successor_proof_3.verify(&old_mmr, &new_mmr));
            }
        }
    }

    #[test]
    fn verification_succeeds_unit() {
        let mut rng: StdRng = SeedableRng::from_seed(
            hex::decode("deadbeef00000000deadbeef00000000deadbeef00000000deadbeef00000000")
                .unwrap()
                .try_into()
                .unwrap(),
        );
        let num_new_leafs = rng.gen_range(0..(1 << 15));
        let old_num_leafs = rng.gen_range(0u64..(u64::MAX >> 55));
        let old_peaks = (0..old_num_leafs.count_ones())
            .map(|_| rng.gen::<Digest>())
            .collect_vec();

        let old_mmr = MmrAccumulator::init(old_peaks, old_num_leafs);
        let mut new_mmr = old_mmr.clone();

        let new_leafs = (0..num_new_leafs)
            .map(|_| rng.gen::<Digest>())
            .collect_vec();

        for &leaf in new_leafs.iter() {
            new_mmr.append(leaf);
        }

        let mmr_successor_proof = MmrSuccessorProof::new_from_batch_append(&old_mmr, &new_leafs);

        assert!(mmr_successor_proof.verify(&old_mmr, &new_mmr));
    }
}
