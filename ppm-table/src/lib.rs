use std::collections::{HashMap, HashSet};
use std::ops::Index;

use bimap::BiHashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PpmTable {
    ppm_table: Vec<Vec<u32>>,
    indices: BiHashMap<String, usize>,
}

impl PpmTable {
    const INDEX_FAIL_PANIC_MESSAGE: &'static str =
        "A PpmTable must correspond to a fully-connected graph.";

    pub fn get_ppm(&self, l: &str, r: &str) -> Option<&u32> {
        let (l_idx, r_idx) = self.table_indices_from_strs(l, r)?;
        Some(&self.ppm_table[l_idx][r_idx])
    }

    pub fn edges(&self) -> impl Iterator<Item = (&str, &str, u32)> {
        self.ppm_table
            .iter()
            .enumerate()
            .flat_map(|(i, v)| v.iter().enumerate().map(move |(j, ppm)| (i, j, ppm)))
            .map(|(i, j, ppm)| {
                let (l, r) = self.strs_from_table_indices(i, j);
                (l, r, *ppm)
            })
    }

    fn table_indices_from_strs(&self, l: &str, r: &str) -> Option<(usize, usize)> {
        let (l, r) = if l < r { (l, r) } else { (r, l) };
        let l_idx = *self.indices.get_by_left(l)?;
        let r_idx = *self.indices.get_by_left(r)? - l_idx - 1;
        Some((l_idx, r_idx))
    }

    fn strs_from_table_indices(&self, l_idx: usize, r_idx: usize) -> (&str, &str) {
        let l = self
            .indices
            .get_by_right(&l_idx)
            .expect(Self::INDEX_FAIL_PANIC_MESSAGE)
            .as_str();
        let r = self
            .indices
            .get_by_right(&(r_idx + l_idx + 1))
            .expect(Self::INDEX_FAIL_PANIC_MESSAGE)
            .as_str();
        (l, r)
    }
}

impl Index<(&str, &str)> for PpmTable {
    type Output = u32;

    fn index(&self, index: (&str, &str)) -> &Self::Output {
        let (l, r) = index;
        self.get_ppm(l, r).expect("no ppm found for strings")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PpmTableBuilder {
    ppms: HashMap<String, HashMap<String, u32>>,
    keys: HashSet<String>,
}

impl PpmTableBuilder {
    pub fn new() -> Self {
        let ppms = HashMap::new();
        let keys = HashSet::new();
        Self { ppms, keys }
    }

    pub fn add_ppm(&mut self, l: String, r: String, ppm: u32) {
        let (l, r) = if l < r { (l, r) } else { (r, l) };
        self.keys.insert(l.clone());
        self.keys.insert(r.clone());
        self.ppms.entry(l).or_default().insert(r, ppm);
    }

    pub fn build(self) -> Result<PpmTable, Self> {
        if !self.data_is_complete() {
            return Err(self);
        }

        let sorted_keys = Self::sorted_keys(self.keys);

        let ppm_table = Self::generate_ppm_table(&sorted_keys, self.ppms);
        let indices = Self::indices_from_sorted_keys(sorted_keys);

        Ok(PpmTable { ppm_table, indices })
    }

    fn data_is_complete(&self) -> bool {
        for l in &self.keys {
            for r in &self.keys {
                if l < r {
                    let Some(l_ppms) = self.ppms.get(l) else {
                        return false;
                    };
                    if !l_ppms.contains_key(r) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn sorted_keys(keys: HashSet<String>) -> Vec<String> {
        let mut key_vec = keys.into_iter().collect::<Vec<_>>();
        key_vec.sort();
        key_vec
    }

    fn generate_ppm_table(
        sorted_keys: &[String],
        ppms: HashMap<String, HashMap<String, u32>>,
    ) -> Vec<Vec<u32>> {
        let mut ppm_table = Self::allocate_ppm_table(sorted_keys.len());
        Self::populate_ppm_table(&mut ppm_table, sorted_keys, ppms);
        ppm_table
    }

    fn indices_from_sorted_keys(sorted_keys: Vec<String>) -> BiHashMap<String, usize> {
        let mut indices = BiHashMap::with_capacity(sorted_keys.len());
        for (i, k) in sorted_keys.into_iter().enumerate() {
            indices.insert(k, i);
        }
        indices
    }

    fn allocate_ppm_table(n: usize) -> Vec<Vec<u32>> {
        let mut outer = Vec::with_capacity(n);
        for i in 0..n {
            outer.push(Vec::with_capacity(n - i - 1));
        }
        outer
    }

    fn populate_ppm_table(
        ppm_table: &mut [Vec<u32>],
        sorted_keys: &[String],
        ppms: HashMap<String, HashMap<String, u32>>,
    ) {
        for (i, l) in sorted_keys.iter().enumerate() {
            for (j, r) in sorted_keys.iter().enumerate() {
                if i < j {
                    ppm_table[i].push(ppms[l][r])
                }
            }
        }
    }
}

impl Default for PpmTableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppm_table_builder_empty() {
        let builder = PpmTableBuilder::default();
        let table = builder.build().expect("Table should be buildable.");
        assert!(table.edges().next().is_none());
    }

    #[test]
    fn test_ppm_table_builder_two_keys() {
        let mut builder = PpmTableBuilder::default();
        builder.add_ppm("a".to_string(), "b".to_string(), 10);
        let table = builder.build().expect("Table should be buildable.");

        assert_eq!(table[("a", "b")], 10);
        assert_eq!(table.edges().collect::<Vec<_>>(), vec![("a", "b", 10)]);
    }

    #[test]
    fn test_ppm_table_builder_missing_similarity() {
        let mut builder = PpmTableBuilder::default();
        builder.add_ppm("a".to_string(), "b".to_string(), 10);
        builder.add_ppm("b".to_string(), "c".to_string(), 20);
        let old_builder = builder.clone();
        let table = builder.build().expect_err("Table should not be buildable.");
        assert_eq!(table, old_builder);
    }

    #[test]
    fn test_ppm_table_builder_three_nodes() {
        let mut builder = PpmTableBuilder::default();
        builder.add_ppm("a".to_string(), "b".to_string(), 10);
        builder.add_ppm("a".to_string(), "c".to_string(), 20);
        builder.add_ppm("b".to_string(), "c".to_string(), 14);
        let table = builder.build().expect("Table should be buildable.");

        let expected = {
            let mut set = HashSet::new();
            set.insert(("a", "b", 10));
            set.insert(("a", "c", 20));
            set.insert(("b", "c", 14));
            set
        };
        assert_eq!(table.edges().collect::<HashSet<_>>(), expected);
    }

    #[test]
    fn test_ppm_table_builder_overwrite() {
        let mut builder = PpmTableBuilder::default();
        builder.add_ppm("a".to_string(), "b".to_string(), 25);
        builder.add_ppm("a".to_string(), "b".to_string(), 16);
        let table = builder.build().expect("Table should be buildable.");

        assert_eq!(table[("a", "b")], 16);
        assert_eq!(table.edges().collect::<Vec<_>>(), vec![("a", "b", 16)]);
    }
}
