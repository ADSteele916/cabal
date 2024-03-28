use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

use crate::clique::{Clique, CliqueExport};

#[derive(Clone, Debug)]
pub struct Cliques<'a> {
    cliques: HashMap<usize, Clique<'a>>,
    base_id: usize,
}

impl<'a> Cliques<'a> {
    pub fn new(base_id: u32) -> Self {
        let cliques = HashMap::new();
        let base_id = base_id as usize;
        Cliques { cliques, base_id }
    }

    pub fn add(&mut self, l: &'a str, r: &'a str, ppm: u32) {
        let lc = self.find_id_of_clique_containing(l);
        let rc = self.find_id_of_clique_containing(r);

        match (lc, rc) {
            (Some(lc), Some(rc)) => {
                if lc != rc {
                    let right_clique = self.cliques.remove(&rc).unwrap();
                    let left_clique = self.cliques.get_mut(&lc).unwrap();
                    left_clique.merge(right_clique);
                    left_clique.add(l, r, ppm);
                } else {
                    self.cliques.get_mut(&lc).unwrap().add(l, r, ppm);
                }
            }
            (Some(lc), None) => {
                self.cliques.get_mut(&lc).unwrap().add(l, r, ppm);
            }
            (None, Some(rc)) => {
                self.cliques.get_mut(&rc).unwrap().add(l, r, ppm);
            }
            (None, None) => {
                self.cliques
                    .insert(self.base_id, Clique::new(l, r, ppm, self.base_id));
                self.base_id += 1
            }
        }
    }

    pub fn export(&self, other: &Self) -> CliquesExport {
        let mut cliques = Vec::new();

        for clique in self.cliques.values() {
            let merged_cliques = Self::merged_cliques(other, clique);
            let added_members = Self::added_members(other, clique);

            if merged_cliques.is_empty() {
                cliques.push(CliquesExportElement::New(clique.export()))
            } else {
                cliques.push(CliquesExportElement::Old {
                    clique: clique.export(),
                    merged: merged_cliques,
                    added: added_members,
                })
            }
        }
        cliques.sort_by(CliquesExportElement::cmp_ppm);
        CliquesExport { cliques }
    }

    fn find_id_of_clique_containing(&self, id: &str) -> Option<usize> {
        self.cliques
            .values()
            .find_map(|c| if c.contains(id) { Some(c.id()) } else { None })
    }

    fn merged_cliques(other: &Self, clique: &Clique) -> Vec<CliqueExport> {
        let merged_clique_ids: HashSet<_> = clique
            .iter()
            .filter_map(|id| {
                other
                    .cliques
                    .values()
                    .find(|&c| c.contains(id))
                    .map(Clique::id)
            })
            .collect();

        let mut merged_cliques: Vec<_> = merged_clique_ids
            .into_iter()
            .map(|id| other.cliques[&id].export())
            .collect();
        merged_cliques.sort_by(CliqueExport::cmp_ppm);

        merged_cliques
    }

    fn added_members(other: &Self, clique: &Clique) -> Vec<String> {
        let mut added_members = Vec::new();
        for id in clique.iter() {
            if !other.cliques.values().any(|c| c.contains(id)) {
                added_members.push(id.to_string());
            }
        }
        added_members.sort();
        added_members
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CliquesExport {
    cliques: Vec<CliquesExportElement>,
}

impl Display for CliquesExport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for clique in &self.cliques {
            write!(f, "{}", clique)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CliquesExportElement {
    New(CliqueExport),
    Old {
        clique: CliqueExport,
        merged: Vec<CliqueExport>,
        added: Vec<String>,
    },
}

impl CliquesExportElement {
    fn cmp_ppm(&self, other: &Self) -> Ordering {
        match (self, other) {
            (CliquesExportElement::New(_), CliquesExportElement::Old { .. }) => Ordering::Greater,
            (CliquesExportElement::Old { .. }, CliquesExportElement::New(_)) => Ordering::Less,
            (_, _) => self.clique().cmp_ppm(other.clique()),
        }
    }

    fn clique(&self) -> &CliqueExport {
        match self {
            CliquesExportElement::New(clique) => clique,
            CliquesExportElement::Old { clique, .. } => clique,
        }
    }
}

impl Display for CliquesExportElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CliquesExportElement::New(clique) => {
                writeln!(f, "New: {}", clique)?;
            }
            CliquesExportElement::Old {
                clique,
                merged,
                added,
            } => {
                writeln!(f, "Old: {}", clique)?;
                if merged.len() > 1 {
                    writeln!(f, "     Absorbed {}:", merged.len())?;
                    for clique in merged {
                        writeln!(f, "          {}", clique)?;
                    }
                }
                if !added.is_empty() {
                    write!(f, "     Added: ")?;
                    for s in added {
                        write!(f, "{} ", s)?;
                    }
                    writeln!(f)?;
                }
            }
        }
        Ok(())
    }
}
