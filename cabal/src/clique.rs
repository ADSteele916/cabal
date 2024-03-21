use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

use petgraph::prelude::*;

#[derive(Clone, Debug)]
pub struct Clique<'a> {
    members: UnGraphMap<&'a str, u32>,
    id: usize,
}

impl<'a> Clique<'a> {
    pub fn new(l: &'a str, r: &'a str, ppm: u32, id: usize) -> Self {
        let members = GraphMap::new();
        let mut new_clique = Self { members, id };
        new_clique.add(l, r, ppm);
        new_clique
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn contains(&self, l: &'a str) -> bool {
        self.members.contains_node(l)
    }

    pub fn add(&mut self, l: &'a str, r: &'a str, ppm: u32) {
        self.members.add_edge(l, r, ppm);
    }

    pub fn iter(&self) -> impl Iterator<Item = &'a str> {
        self.members.nodes().collect::<Vec<_>>().into_iter()
    }

    pub fn merge(&mut self, o: Clique<'a>) {
        for (l, r, ppm) in o.members.all_edges() {
            self.add(l, r, *ppm)
        }
    }

    pub fn core(&self) -> &'a str {
        let mut min_difference_and_key = None;

        for node in self.members.nodes() {
            let max = self
                .members
                .edges(node)
                .map(|(_, _, ppm)| *ppm)
                .max()
                .unwrap_or(0);

            min_difference_and_key = match min_difference_and_key {
                Some((old_min_difference, old_node)) => {
                    if (max < old_min_difference)
                        || (max == old_min_difference) && (node < old_node)
                    {
                        Some((max, node))
                    } else {
                        Some((old_min_difference, old_node))
                    }
                }
                None => Some((max, node)),
            };
        }

        min_difference_and_key.unwrap().1
    }

    pub fn export(&self) -> CliqueExport {
        let core = self.core().to_string();
        let non_core_members = self
            .members
            .nodes()
            .filter(|n| *n != core)
            .map(|n| n.to_string())
            .collect();
        let max_ppm = self.max_ppm();

        CliqueExport {
            core,
            non_core_members,
            max_ppm,
        }
    }

    fn max_ppm(&self) -> u32 {
        self.members
            .all_edges()
            .map(|(_, _, ppm)| *ppm)
            .max()
            .unwrap_or(0)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CliqueExport {
    core: String,
    non_core_members: Vec<String>,
    max_ppm: u32,
}

impl CliqueExport {
    pub fn cmp_ppm(&self, other: &Self) -> Ordering {
        self.max_ppm.cmp(&other.max_ppm)
    }
}

impl Display for CliqueExport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut sortable = self.non_core_members.clone();
        sortable.sort();
        sortable.insert(0, self.core.clone());

        write!(
            f,
            "[{}]",
            sortable
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )?;

        write!(
            f,
            " max%: {}.{}",
            self.max_ppm / 10000,
            (self.max_ppm % 10000) / 1000
        )?;

        Ok(())
    }
}
