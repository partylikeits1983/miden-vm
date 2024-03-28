
## std::collections::mmr
| Procedure | Description |
| ----------- | ------------- |
| get | Loads the leaf at the absolute `pos` in the MMR.<br /><br />This MMR implementation supports only u32 positions.<br /><br />Stack transition:<br />Input: [pos, mmr_ptr, ...]<br />Output: [N, ...] where `N` is the leaf and `R` is the MMR peak that owns the leaf.<br /><br />Cycles: 115<br /> |
| num_leaves_to_num_peaks | Given the num_leaves of a MMR returns the num_peaks.<br /><br />Input: [num_leaves, ...]<br />Output: [num_peaks, ...]<br />Cycles: 69<br /> |
| num_peaks_to_message_size | Given the num_peaks of a MMR, returns the hasher state size after accounting<br />for the required padding.<br /><br />Input: [num_peaks, ...]<br />Output: [len, ...]<br />Cycles: 17<br /> |
| unpack | Load the MMR peak data based on its hash.<br /><br />Input: [HASH, mmr_ptr, ...]<br />Output: [...]<br /><br />Where:<br />- HASH: is the MMR peak hash, the hash is expected to be padded to an even<br />length and to have a minimum size of 16 elements<br />- The advice map must contain a key with HASH, and its value is<br />`num_leaves \|\| hash_data`, and hash_data is the data used to computed `HASH`<br />- mmt_ptr: the memory location where the MMR data will be written to,<br />starting with the MMR forest (its total leaves count) followed by its peaks<br /><br />Cycles: 162 + 9 * extra_peak_pair cycles<br />where `extra_peak` is the number of peak pairs in addition to the first<br />16, i.e. `round_up((num_of_peaks - 16) / 2)`<br /> |
| pack | Computes the hash of the given MMR and copies it to the Advice Map using its hash as a key.<br /><br />Input: [mmr_ptr, ...]<br />Output: [HASH, ...]<br />Cycles: 128 + 3 * num_peaks<br /> |
| add | Adds a new element to the MMR.<br /><br />This will update the MMR peaks in the VM's memory and the advice provider<br />with any merged nodes.<br /><br />Input: [EL, mmr_ptr, ...]<br />Output: [...]<br />Cycles: 144 + 39 * peak_merges<br /> |
