// Copyright (C) 2022 CSC - IT Center for Science Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use lazy_static::lazy_static;
use pyo3::prelude::*;
use pyo3::types::*;
use pyo3::wrap_pyfunction;
use regex::Regex;
use scan_fmt::scan_fmt_some;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Molecule {
    #[pyo3(get, set)]
    mol_name: String,
    #[pyo3(get, set)]
    num_atoms: Option<usize>,
    #[pyo3(get, set)]
    num_bonds: Option<usize>,
    #[pyo3(get, set)]
    num_subst: Option<usize>,
    #[pyo3(get, set)]
    num_feat: Option<usize>,
    #[pyo3(get, set)]
    num_sets: Option<usize>,
    #[pyo3(get, set)]
    mol_type: Option<String>,
    #[pyo3(get, set)]
    charge_type: Option<String>,
    #[pyo3(get, set)]
    status_bits: Option<String>,
    #[pyo3(get, set)]
    mol_comment: Option<String>,
}

impl Molecule {
    fn new() -> Molecule {
        return Molecule {
            mol_name: String::new(),
            num_atoms: None,
            num_bonds: None,
            num_subst: None,
            num_feat: None,
            num_sets: None,
            mol_type: None,
            charge_type: None,
            status_bits: None,
            mol_comment: None,
        };
    }
    fn read_nums(&mut self, line: &str) {
        lazy_static! {
            static ref NUMBER: Regex = Regex::new(r"(\d+)").expect("Failed to create a regex");
        }

        for (index, capture) in NUMBER.captures_iter(line).enumerate() {
            let number = capture
                .get(1)
                .expect("Something went really wrong in the second line of the MOLECULE section")
                .as_str()
                .parse::<usize>()
                .ok();
            match index {
                0 => self.num_atoms = number,
                1 => self.num_bonds = number,
                2 => self.num_subst = number,
                3 => self.num_feat = number,
                4 => self.num_sets = number,
                _ => continue,
            }
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Atom {
    #[pyo3(get, set)]
    atom_id: usize,
    #[pyo3(get, set)]
    atom_name: String,
    #[pyo3(get, set)]
    x: f64,
    #[pyo3(get, set)]
    y: f64,
    #[pyo3(get, set)]
    z: f64,
    #[pyo3(get, set)]
    atom_type: String,
    #[pyo3(get, set)]
    subst_id: Option<usize>,
    #[pyo3(get, set)]
    subst_name: Option<String>,
    #[pyo3(get, set)]
    charge: Option<f64>,
    #[pyo3(get, set)]
    status_bit: Option<String>,
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Bond {
    #[pyo3(get, set)]
    bond_id: usize,
    #[pyo3(get, set)]
    origin_atom_id: usize,
    #[pyo3(get, set)]
    target_atom_id: usize,
    #[pyo3(get, set)]
    bond_type: String,
    #[pyo3(get, set)]
    status_bit: Option<String>,
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Substructure {
    #[pyo3(get, set)]
    subst_id: usize,
    #[pyo3(get, set)]
    subst_name: String,
    #[pyo3(get, set)]
    root_atom: usize,
    #[pyo3(get, set)]
    subst_type: Option<String>,
    #[pyo3(get, set)]
    dict_type: Option<i64>,
    #[pyo3(get, set)]
    chain: Option<String>,
    #[pyo3(get, set)]
    sub_type: Option<String>,
    #[pyo3(get, set)]
    inter_bonds: Option<usize>,
    #[pyo3(get, set)]
    status: Option<String>,
    #[pyo3(get, set)]
    comment: Option<String>,
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Mol2 {
    #[pyo3(get, set)]
    molecule: Option<Molecule>,
    #[pyo3(get, set)]
    atom: Vec<Atom>,
    #[pyo3(get, set)]
    bond: Vec<Bond>,
    #[pyo3(get, set)]
    substructure: Vec<Substructure>,
}

impl Mol2 {
    fn new() -> Mol2 {
        return Mol2 {
            molecule: None,
            atom: Vec::new(),
            bond: Vec::new(),
            substructure: Vec::new(),
        };
    }
}

#[pymethods]
impl Mol2 {
    fn to_json(&self) -> String {
        let json_str: String =
            serde_json::to_string(self).expect("Failed to translate mol2 into json format");
        return json_str;
    }
    fn serialized(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let json = PyModule::import(py, "json").expect("Failed to import python json module");
            let json = json
                .getattr("loads")?
                .call1((self.to_json(),))?
                .downcast::<PyDict>()?
                .to_object(py);
            Ok(json)
        })
    }
}

fn read_molecule_section(nline: usize, line: &str, mol2: &mut Mol2) {
    if line.len() == 0 {
        return;
    }
    match nline {
        0 => mol2.molecule.get_or_insert(Molecule::new()).mol_name = line.to_owned(),
        1 => mol2
            .molecule
            .as_mut()
            .expect("You seem to be reading lines in the wrong order...?")
            .read_nums(line),
        2 => {
            mol2.molecule.get_or_insert(Molecule::new()).mol_type =
                Some(line.split_whitespace().next().get_or_insert("").to_owned())
        }
        3 => {
            mol2.molecule.get_or_insert(Molecule::new()).charge_type =
                Some(line.split_whitespace().next().get_or_insert("").to_owned())
        }
        4 => {
            mol2.molecule.get_or_insert(Molecule::new()).status_bits =
                Some(line.split_whitespace().next().get_or_insert("").to_owned())
        }
        5 => mol2.molecule.get_or_insert(Molecule::new()).mol_comment = Some(line.to_owned()),
        _ => return,
    }
}

fn read_atom_section(line: &str, mol2: &mut Mol2) {
    if line.len() == 0 {
        return;
    }
    let (atom_id, atom_name, x, y, z, atom_type, subst_id, subst_name, charge, status_bit) = scan_fmt_some!(
        line,
        r" {d} {} {f} {f} {f} {} {d} {} {f} {}",
        usize,
        String,
        f64,
        f64,
        f64,
        String,
        usize,
        String,
        f64,
        String
    );
    let atom = Atom {
        atom_id: atom_id.expect("Failed to get atom id from the atom section line"),
        atom_name: atom_name.expect("Failed to get atom name from the atom section line"),
        x: x.expect("Failed to get atom x from the atom section line"),
        y: y.expect("Failed to get atom y from the atom section line"),
        z: z.expect("Failed to get atom z from the atom section line"),
        atom_type: atom_type.expect("Failed to get atom type from the atom section line"),
        subst_id,
        subst_name,
        charge,
        status_bit,
    };
    mol2.atom.push(atom);
}

fn read_bond_section(line: &str, mol2: &mut Mol2) {
    if line.len() == 0 {
        return;
    }
    let (bond_id, origin_atom_id, target_atom_id, bond_type, status_bit) = scan_fmt_some!(
        line,
        r" {d} {d} {d} {} {}",
        usize,
        usize,
        usize,
        String,
        String
    );
    let bond = Bond {
        bond_id: bond_id.expect("Failed to get bond id from the bond section line"),
        origin_atom_id: origin_atom_id
            .expect("Failed to get origin atom id from the bond section line"),
        target_atom_id: target_atom_id
            .expect("Failed to get target atom id from the bond section line"),
        bond_type: bond_type.expect("Failed to get bond type from the bond section line"),
        status_bit,
    };
    mol2.bond.push(bond);
}

fn read_substructure_section(line: &str, mol2: &mut Mol2) {
    if line.len() == 0 {
        return;
    }
    let (
        subst_id,
        subst_name,
        root_atom,
        subst_type,
        dict_type,
        chain,
        sub_type,
        inter_bonds,
        status,
        comment,
    ) = scan_fmt_some!(
        line,
        r" {d} {} {d} {} {d} {} {} {d} {} {/.*/}",
        usize,
        String,
        usize,
        String,
        i64,
        String,
        String,
        usize,
        String,
        String
    );
    let substructure = Substructure {
        subst_id: subst_id.expect("Failed to get subst_id from a substructure section line"),
        subst_name: subst_name.expect("Failed to get subst_name from a substructure section line"),
        root_atom: root_atom.expect("Failed to get root atom from a substructure section line"),
        subst_type,
        dict_type,
        chain,
        sub_type,
        inter_bonds,
        status,
        comment,
    };
    mol2.substructure.push(substructure);
}

#[pyfunction]
fn read_file(filename: &str) -> PyResult<Vec<Mol2>> {
    let file = File::open(filename).expect("Failed to open the input file");
    let reader = BufReader::new(file);
    let mut section_name: String = String::new();
    let mut section_index: usize = 0;
    let mut mol2: Vec<Mol2> = Vec::new();
    let mut entry = Mol2::new();
    for (index, line) in reader.lines().enumerate() {
        let line =
            line.expect(&format!("Failed to read {} line from the input file", index + 1)[..]);

        lazy_static! {
            static ref NEW_SECTION: Regex =
                Regex::new(r"^@<TRIPOS>(\w+)").expect("Failed to create a regex");
        }
        if NEW_SECTION.is_match(&line) {
            section_name = NEW_SECTION
                .captures(&line)
                .expect("captures failed on new section")
                .get(1)
                .expect("There was a match for new section but name seems to be missing...?")
                .as_str()
                .to_owned();
            section_index = index;
            if section_name == "MOLECULE" && entry.molecule.is_some() {
                mol2.push(entry);
                entry = Mol2::new();
            }
        } else if section_name != String::new() {
            let subsection_index = index - section_index - 1;
            match &section_name[..] {
                "MOLECULE" => read_molecule_section(subsection_index, &line, &mut entry),
                "ATOM" => read_atom_section(&line, &mut entry),
                "BOND" => read_bond_section(&line, &mut entry),
                "SUBSTRUCTURE" => read_substructure_section(&line, &mut entry),
                _ => continue,
            };
        }
    }
    mol2.push(entry); // if we are just at the end of the file

    return Ok(mol2);
}

#[pyfunction]
fn read_file_serialized(filename: &str) -> PyResult<Vec<PyObject>> {
    let mol2_list = read_file(filename).expect("Failed to read mol2 file");
    let mut result: Vec<PyObject> = Vec::new();
    for entry in &mol2_list {
        result.push(
            entry
                .serialized()
                .expect("Failed to serialize mol2 entry..."),
        );
    }
    return Ok(result);
}

#[pymodule]
fn serde_mol2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Molecule>()?;
    m.add_class::<Atom>()?;
    m.add_class::<Bond>()?;
    m.add_class::<Substructure>()?;
    m.add_class::<Mol2>()?;
    m.add_wrapped(wrap_pyfunction!(read_file))?;
    m.add_wrapped(wrap_pyfunction!(read_file_serialized))?;

    Ok(())
}
