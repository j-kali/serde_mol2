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

use bincode;
use lazy_static::lazy_static;
use pyo3::prelude::*;
use pyo3::types::*;
use pyo3::wrap_pyfunction;
use regex::Regex;
use rusqlite;
use scan_fmt::scan_fmt_some;
use serde::{Deserialize, Serialize};
use serde_json;
use std::any::TypeId;
use std::fs::File;
use std::io::{BufRead, BufReader};
use zstd;

type IdInt = u16;
type ChargeFloat = f32;
type CoordFloat = f64;

fn my_append_value<T: 'static>(line: &mut String, value: &Option<T>)
where
    T: std::fmt::Display,
{
    if line.len() > 0 {
        line.push_str(", ");
    }
    match value {
        Some(c) => {
            if TypeId::of::<T>() == TypeId::of::<String>() {
                line.push_str(&format!("'{}'", c)[..]);
            } else {
                line.push_str(&format!("{}", c)[..]);
            }
        }
        _ => line.push_str("NULL"),
    };
}

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
    fn values(&self) -> String {
        let mut values: String = String::new();
        my_append_value(&mut values, &Some(self.mol_name.clone()));
        my_append_value(&mut values, &self.num_atoms);
        my_append_value(&mut values, &self.num_bonds);
        my_append_value(&mut values, &self.num_subst);
        my_append_value(&mut values, &self.num_feat);
        my_append_value(&mut values, &self.num_sets);
        my_append_value(&mut values, &self.mol_type);
        my_append_value(&mut values, &self.charge_type);
        my_append_value(&mut values, &self.status_bits);
        my_append_value(&mut values, &self.mol_comment);
        return values;
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
    atom_id: IdInt,
    #[pyo3(get, set)]
    atom_name: String,
    #[pyo3(get, set)]
    x: CoordFloat,
    #[pyo3(get, set)]
    y: CoordFloat,
    #[pyo3(get, set)]
    z: CoordFloat,
    #[pyo3(get, set)]
    atom_type: String,
    #[pyo3(get, set)]
    subst_id: Option<IdInt>,
    #[pyo3(get, set)]
    subst_name: Option<String>,
    #[pyo3(get, set)]
    charge: Option<ChargeFloat>,
    #[pyo3(get, set)]
    status_bit: Option<String>,
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Bond {
    #[pyo3(get, set)]
    bond_id: IdInt,
    #[pyo3(get, set)]
    origin_atom_id: IdInt,
    #[pyo3(get, set)]
    target_atom_id: IdInt,
    #[pyo3(get, set)]
    bond_type: String,
    #[pyo3(get, set)]
    status_bit: Option<String>,
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Substructure {
    #[pyo3(get, set)]
    subst_id: IdInt,
    #[pyo3(get, set)]
    subst_name: String,
    #[pyo3(get, set)]
    root_atom: IdInt,
    #[pyo3(get, set)]
    subst_type: Option<String>,
    #[pyo3(get, set)]
    dict_type: Option<i64>,
    #[pyo3(get, set)]
    chain: Option<String>,
    #[pyo3(get, set)]
    sub_type: Option<String>,
    #[pyo3(get, set)]
    inter_bonds: Option<IdInt>,
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
        IdInt,
        String,
        CoordFloat,
        CoordFloat,
        CoordFloat,
        String,
        IdInt,
        String,
        ChargeFloat,
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
        IdInt,
        IdInt,
        IdInt,
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
        IdInt,
        String,
        IdInt,
        String,
        i64,
        String,
        String,
        IdInt,
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

fn create_table(db: &rusqlite::Connection) -> Result<(), ()> {
    match db.execute("CREATE TABLE structures (id INTEGER PRIMARY KEY, mol_name TEXT, num_atoms INTEGER, num_bonds INTEGER, num_subst INTEGER, num_feat INTEGER, num_sets INTEGER, mol_type TEXT, charge_type TEXT, status_bits TEXT, mol_comment TEXT, atom BLOB, bond BLOB, substructure BLOB, compression INTEGER)", []) {
        Ok(_) => Ok(()),
        _ => Err(()),
    }
}

fn get_db(filename: &str) -> rusqlite::Connection {
    let db = rusqlite::Connection::open(filename).expect("Connection to the db failed");
    let _ = create_table(&db);
    return db;
}

fn db_entry_insert(
    mol2_entry: &Mol2,
    db: &rusqlite::Connection,
    compression: i32,
) -> Result<(), ()> {
    let mut insert_cmd: String = String::new();
    insert_cmd.push_str("INSERT INTO structures (mol_name, num_atoms, num_bonds, num_subst, num_feat, num_sets, mol_type, charge_type, status_bits, mol_comment, atom, bond, substructure, compression)");
    insert_cmd.push_str(
        &format!(
            " VALUES ({}",
            mol2_entry.molecule.as_ref().unwrap().values()
        )[..],
    );
    insert_cmd.push_str(", ?1, ?2, ?3");
    // Handle compression levels
    let mut compression_level = compression;
    if compression_level > 9 {
        compression_level = 9;
    }
    insert_cmd.push_str(&format!(", {})", compression_level)[..]);
    let mut atom = bincode::serialize(&mol2_entry.atom).expect("Failed to serialize into binary");
    let mut bond = bincode::serialize(&mol2_entry.bond).expect("Failed to serialize into binary");
    let mut subs =
        bincode::serialize(&mol2_entry.substructure).expect("Failed to serialize into binary");
    if compression_level > 0 {
        atom = zstd::block::Compressor::new()
            .compress(&atom, compression_level)
            .expect("Compression failed");
        bond = zstd::block::Compressor::new()
            .compress(&bond, compression_level)
            .expect("Compression failed");
        subs = zstd::block::Compressor::new()
            .compress(&subs, compression_level)
            .expect("Compression failed");
    }
    db.execute(&insert_cmd, rusqlite::params![atom, bond, subs])
        .expect("Failed to insert data to db");
    return Ok(());
}

#[pyfunction]
fn db_insert(mol2_list: Vec<Mol2>, filename: &str, compression: i32) -> PyResult<()> {
    let db = get_db(filename);
    let _ = create_table(&db);
    for entry in mol2_list.iter() {
        let _ = db_entry_insert(entry, &db, compression);
    }
    return Ok(());
}

#[pyfunction]
fn read_db_all(filename: &str) -> PyResult<Vec<Mol2>> {
    let db = get_db(filename);
    let mut stmt = db.prepare("SELECT mol_name, num_atoms, num_bonds, num_subst, num_feat, num_sets, mol_type, charge_type, status_bits, mol_comment, atom, bond, substructure, compression FROM structures").expect("Failed to fetch from the database");
    let structure_iter = stmt
        .query_map([], |row| {
            let compression: i32 = row.get(13).unwrap();
            let mut atom: Vec<u8> = row.get(10).unwrap();
            let mut bond: Vec<u8> = row.get(11).unwrap();
            let mut subs: Vec<u8> = row.get(12).unwrap();
            if compression > 0 {
                atom = zstd::block::Decompressor::new()
                    .decompress(&atom, usize::MAX)
                    .expect("Failed to decompress");
                bond = zstd::block::Decompressor::new()
                    .decompress(&bond, usize::MAX)
                    .expect("Failed to decompress");
                subs = zstd::block::Decompressor::new()
                    .decompress(&subs, usize::MAX)
                    .expect("Failed to decompress");
            }
            let atom: Vec<Atom> =
                bincode::deserialize(&atom).expect("Failed to deserialize &[u8] to Atom");
            let bond: Vec<Bond> =
                bincode::deserialize(&bond).expect("Failed to deserialize &[u8] to Bond");
            let substructure: Vec<Substructure> =
                bincode::deserialize(&subs).expect("Failed to deserialize &[u8] to Substructure");
            Ok(Mol2 {
                molecule: Some(Molecule {
                    mol_name: row.get(0).unwrap(),
                    num_atoms: row.get(1).unwrap(),
                    num_bonds: row.get(2).unwrap(),
                    num_subst: row.get(3).unwrap(),
                    num_feat: row.get(4).unwrap(),
                    num_sets: row.get(5).unwrap(),
                    mol_type: row.get(6).unwrap(),
                    charge_type: row.get(7).unwrap(),
                    status_bits: row.get(8).unwrap(),
                    mol_comment: row.get(9).unwrap(),
                }),
                atom,
                bond,
                substructure,
            })
        })
        .expect("Failed to fetch exact numbers from db");
    let mut mol2_list: Vec<Mol2> = Vec::new();
    for structure in structure_iter {
        mol2_list.push(structure.expect("Failed to get structure after successful extraction...?"));
    }

    Ok(mol2_list)
}

#[pyfunction]
fn read_db_all_serialized(filename: &str) -> PyResult<Vec<PyObject>> {
    let mol2_list = read_db_all(filename).expect("Failed to read mol2 db");
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
    m.add_wrapped(wrap_pyfunction!(db_insert))?;
    m.add_wrapped(wrap_pyfunction!(read_db_all))?;
    m.add_wrapped(wrap_pyfunction!(read_db_all_serialized))?;

    Ok(())
}
