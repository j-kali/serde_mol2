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

use pyo3::prelude::*;
use pyo3::types::*;
use pyo3::wrap_pyfunction;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;

type IdInt = u16;
type ChargeFloat = f32;
type CoordFloat = f64;

// Using a rather large buffer but for our applications should be fine.
static DECOMPRESSOR_BUFFER: usize = 100 * 1024 * 1024;

fn write_string(text: &str, filename: &str, append: bool) {
    // Helper function to standardize writing strings to files
    // Input:
    //     text: string to write
    //     filename: path to a file to write to
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(!append)
        .append(append)
        .open(filename)
        .expect("Failed to open tmp file");
    file.write_all(text.as_bytes())
        .expect("Failed to write to a mol2 file");
}

// Struct for holding data from MOLECULE sections
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
        Molecule {
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
        }
    }
    fn read_nums(&mut self, line: &str) {
        // Read the second line of the section
        for (index, word) in line.split_whitespace().enumerate() {
            let number = word.parse::<usize>().ok();
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
    fn as_string(&self) -> String {
        // Show object as a mol2 section string
        let mut text = "@<TRIPOS>MOLECULE\n".to_owned();

        for nline in 0..6 {
            match nline {
                0 => text.push_str(&self.mol_name),
                1 => {
                    for nnum in 0..5 {
                        let number = match nnum {
                            0 => self.num_atoms,
                            1 => self.num_bonds,
                            2 => self.num_subst,
                            3 => self.num_feat,
                            4 => self.num_sets,
                            _ => continue,
                        };
                        if number.is_none() {
                            break;
                        } else {
                            if nnum > 0 {
                                text.push(' ');
                            }
                            text.push_str(&format!("{}", number.unwrap())[..]);
                        }
                    }
                }
                2 => text.push_str(self.mol_type.as_ref().unwrap_or(&"".to_owned())),
                3 => text.push_str(self.charge_type.as_ref().unwrap_or(&"".to_owned())),
                4 => {
                    if self.status_bits.is_none() && self.mol_comment.is_some() {
                        text.push_str("****");
                    } else {
                        text.push_str(self.status_bits.as_ref().unwrap_or(&"".to_owned()));
                    }
                }
                5 => text.push_str(self.mol_comment.as_ref().unwrap_or(&"".to_owned())),
                _ => continue,
            }
            // Add a newline at the end of every line
            text.push('\n');
        }

        text
    }
}

// Struct holding data for a single atom entry in the ATOM section of the mol2 format
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

impl Atom {
    fn as_string(&self) -> String {
        // Show atom entry as a string in mol2 ATOM section
        let mut text = String::new();

        text.push_str(
            &format!(
                "{} {} {} {} {} {}",
                self.atom_id, self.atom_name, self.x, self.y, self.z, self.atom_type
            )[..],
        );

        for n in 0..4 {
            if match n {
                0 => self.subst_id.is_none(),
                1 => self.subst_name.is_none(),
                2 => self.charge.is_none(),
                3 => self.status_bit.is_none(),
                _ => continue,
            } {
                break;
            }
            text.push(' ');
            match n {
                0 => text.push_str(&format!("{}", self.subst_id.as_ref().unwrap())[..]),
                1 => text.push_str(self.subst_name.as_ref().unwrap()),
                2 => text.push_str(&format!("{}", self.charge.as_ref().unwrap())[..]),
                3 => text.push_str(self.status_bit.as_ref().unwrap()),
                _ => continue,
            }
        }
        text.push('\n');

        text
    }
}

// Struct holding data for a single entry in BOND section of the mol2 file
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

impl Bond {
    fn as_string(&self) -> String {
        // Show bond entry as a string in mol2 BOND section
        let mut text = String::new();

        text.push_str(
            &format!(
                "{} {} {} {}",
                self.bond_id, self.origin_atom_id, self.target_atom_id, self.bond_type
            )[..],
        );

        if self.status_bit.is_some() {
            text.push_str(&format!(" {}", self.status_bit.as_ref().unwrap())[..]);
        }
        text.push('\n');

        text
    }
}

// Struct holding data for a single entry in SUBSTRUCTURE section of the mol2 file
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

impl Substructure {
    fn as_string(&self) -> String {
        // Show substructure entry as a string in mol2 SUBSTRUCTURE section
        let mut text = String::new();

        text.push_str(&format!("{} {} {}", self.subst_id, self.subst_name, self.root_atom)[..]);

        for n in 0..7 {
            if match n {
                0 => self.subst_type.is_none(),
                1 => self.dict_type.is_none(),
                2 => self.chain.is_none(),
                3 => self.sub_type.is_none(),
                4 => self.inter_bonds.is_none(),
                5 => self.status.is_none(),
                6 => self.comment.is_none(),
                _ => continue,
            } {
                break;
            }
            text.push(' ');
            match n {
                0 => text.push_str(self.subst_type.as_ref().unwrap()),
                1 => text.push_str(&format!("{}", self.dict_type.as_ref().unwrap())[..]),
                2 => text.push_str(self.chain.as_ref().unwrap()),
                3 => text.push_str(self.sub_type.as_ref().unwrap()),
                4 => text.push_str(&format!("{}", self.inter_bonds.as_ref().unwrap())[..]),
                5 => text.push_str(self.status.as_ref().unwrap()),
                6 => text.push_str(self.comment.as_ref().unwrap()),
                _ => continue,
            }
        }
        text.push('\n');

        text
    }
}

// Struct for holding data for a single structure out of a mol2 file
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mol2 {
    #[pyo3(get, set)]
    molecule: Option<Molecule>,
    #[pyo3(get, set)]
    atom: Vec<Atom>,
    #[pyo3(get, set)]
    bond: Vec<Bond>,
    #[pyo3(get, set)]
    substructure: Vec<Substructure>,
    #[pyo3(get, set)]
    desc: Option<String>,
}

impl Mol2 {
    fn new(desc: &str) -> Mol2 {
        let description = match desc.is_empty() {
            true => None,
            false => Some(desc.to_owned()),
        };
        Mol2 {
            molecule: None,
            atom: Vec::new(),
            bond: Vec::new(),
            substructure: Vec::new(),
            desc: description,
        }
    }
    fn add_comment(&mut self, comment: &str) {
        if self.molecule.is_none() || comment.is_empty() {
            return;
        }
        if self.molecule.as_ref().unwrap().mol_comment.is_some() {
            self.molecule
                .as_mut()
                .unwrap()
                .mol_comment
                .as_mut()
                .unwrap()
                .push_str("; ");
        } else {
            self.molecule.as_mut().unwrap().mol_comment = Some(String::new());
        }
        self.molecule
            .as_mut()
            .unwrap()
            .mol_comment
            .as_mut()
            .unwrap()
            .push_str(comment);
    }
}

#[pymethods]
impl Mol2 {
    fn to_json(&self) -> String {
        // Convert to a json string, useful in some cases. But in most cases one should probably use _serialized version of the read function
        let json_str: String =
            serde_json::to_string(self).expect("Failed to translate mol2 into json format");
        json_str
    }
    fn as_string(&self) -> String {
        // Show whole structure in a mol2 compliant string
        let mut text = String::new();

        if self.molecule.is_none() {
            return text;
        }

        text.push_str(&self.molecule.as_ref().unwrap().as_string());
        // We should probably have a generic function for these section thingies...
        if !self.atom.is_empty() {
            text.push_str("@<TRIPOS>ATOM\n");
            for entry in &self.atom {
                text.push_str(&entry.as_string());
            }
            text.push('\n');
        }
        if !self.bond.is_empty() {
            text.push_str("@<TRIPOS>BOND\n");
            for entry in &self.bond {
                text.push_str(&entry.as_string());
            }
            text.push('\n');
        }
        if !self.substructure.is_empty() {
            text.push_str("@<TRIPOS>SUBSTRUCTURE\n");
            for entry in &self.substructure {
                text.push_str(&entry.as_string());
            }
            text.push('\n');
        }

        text
    }
    #[args(filename, append = "false")]
    fn write_mol2(&self, filename: &str, append: bool) {
        // Write structure as a mol2 file
        write_string(&self.as_string(), filename, append);
    }
    fn serialized(&self) -> PyResult<PyObject> {
        // give a serialized version of the structure rather than binary form
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
    // Reading lines from a MOLECULE section
    // Input:
    //     nline: line number within the section
    //     line: line string to parse
    //     mol2: structure to update
    if line.is_empty() {
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
        _ => {}
    }
}

fn read_atom_section(line: &str, mol2: &mut Mol2) {
    // Reading lines from an ATOM section
    // Input:
    //     line: line string to parse
    //     mol2: structure to update
    if line.is_empty() {
        return;
    }

    let mut atom = Atom {
        atom_id: 0,
        atom_name: String::new(),
        x: 0.0,
        y: 0.0,
        z: 0.0,
        atom_type: String::new(),
        subst_id: None,
        subst_name: None,
        charge: None,
        status_bit: None,
    };

    for (index, word) in line.split_whitespace().enumerate() {
        match index {
            0 => atom.atom_id = word.parse::<IdInt>().expect("Failed to parse atom id"),
            1 => atom.atom_name.push_str(word),
            2 => atom.x = word.parse::<CoordFloat>().expect("Failed to parse atom x"),
            3 => atom.y = word.parse::<CoordFloat>().expect("Failed to parse atom y"),
            4 => atom.z = word.parse::<CoordFloat>().expect("Failed to parse atom z"),
            5 => atom.atom_type.push_str(word),
            6 => {
                atom.subst_id = Some(
                    word.parse::<IdInt>()
                        .expect("Failed to parse atom subst_id"),
                )
            }
            7 => atom.subst_name = Some(word.to_owned()),
            8 => {
                atom.charge = Some(
                    word.parse::<ChargeFloat>()
                        .expect("Failed to parse atom charge"),
                )
            }
            9 => atom.status_bit = Some(word.to_owned()),
            _ => continue,
        };
    }

    mol2.atom.push(atom);
}

fn read_bond_section(line: &str, mol2: &mut Mol2) {
    // Reading lines from a BOND section
    // Input:
    //     line: line string to parse
    //     mol2: structure to update
    if line.is_empty() {
        return;
    }
    let mut bond = Bond {
        bond_id: 0,
        origin_atom_id: 0,
        target_atom_id: 0,
        bond_type: String::new(),
        status_bit: None,
    };
    for (index, word) in line.split_whitespace().enumerate() {
        match index {
            0 => {
                bond.bond_id = word
                    .parse::<IdInt>()
                    .expect("Failed to get bond id from the bond section line")
            }
            1 => {
                bond.origin_atom_id = word
                    .parse::<IdInt>()
                    .expect("Failed to get origin atom id from the bond section line")
            }
            2 => {
                bond.target_atom_id = word
                    .parse::<IdInt>()
                    .expect("Failed to get target atom id from the bond section line")
            }
            3 => bond.bond_type.push_str(word),
            4 => bond.status_bit = Some(word.to_owned()),
            _ => continue,
        };
    }
    mol2.bond.push(bond);
}

fn read_substructure_section(line: &str, mol2: &mut Mol2) {
    // Reading lines from a SUBSTRUCTURE section
    // Input:
    //     line: line string to parse
    //     mol2: structure to update
    if line.is_empty() {
        return;
    }
    let mut comment = String::new();
    let mut subs = Substructure {
        subst_id: 0,
        subst_name: String::new(),
        root_atom: 0,
        subst_type: None,
        dict_type: None,
        chain: None,
        sub_type: None,
        inter_bonds: None,
        status: None,
        comment: None,
    };
    for (index, word) in line.split_whitespace().enumerate() {
        match index {
            0 => {
                subs.subst_id = word
                    .parse::<IdInt>()
                    .expect("Failed to get subst_id from a substructure section line")
            }
            1 => subs.subst_name = word.to_owned(),
            2 => {
                subs.root_atom = word
                    .parse::<IdInt>()
                    .expect("Failed to get root atom from a substructure section line")
            }
            3 => subs.subst_type = Some(word.to_owned()),
            4 => subs.dict_type = word.parse::<i64>().ok(),
            5 => subs.chain = Some(word.to_owned()),
            6 => subs.sub_type = Some(word.to_owned()),
            7 => subs.inter_bonds = word.parse::<IdInt>().ok(),
            8 => subs.status = Some(word.to_owned()),
            9 => comment.push_str(word),
            _ => continue,
        };
    }
    if !comment.is_empty() {
        subs.comment = Some(comment);
    }
    mol2.substructure.push(subs);
}

fn create_table(db: &rusqlite::Connection) -> Result<(), ()> {
    // Create a table in the database
    // Input:
    //     db: connection to the database
    match db.execute("CREATE TABLE structures (id INTEGER PRIMARY KEY, mol_name TEXT, num_atoms INTEGER, num_bonds INTEGER, num_subst INTEGER, num_feat INTEGER, num_sets INTEGER, mol_type TEXT, charge_type TEXT, status_bits TEXT, mol_comment TEXT, atom BLOB, bond BLOB, substructure BLOB, extras BLOB, compression INTEGER, desc TEXT)", []) {
        Ok(_) => Ok(()),
        _ => Err(()),
    }
}

fn get_db(filename: &str, in_mem: bool) -> rusqlite::Connection {
    // Get a connection to the database
    // Input:
    //     filename: location on the filesystem
    //     in_mem: should we try to make a temporary copy of the database on faster filesystem before opening?
    //
    // in_mem by default will attempt to copy a database to /dev/shm
    // and work there, that is why it is called 'in_mem'. In the
    // future we probably want to allow other temporary folders to
    // allow for example work on NVMe
    let mut real_path = filename.to_owned();
    if in_mem {
        real_path = "/dev/shm/tmp.sqlite".to_owned();
        if std::path::Path::new(&real_path).exists() {
            std::fs::remove_file(&real_path)
                .expect("Failed to delete existing db file on the shm device...");
        }
        if std::path::Path::new(filename).exists() && std::fs::copy(filename, &real_path).is_err() {
            real_path = filename.to_owned();
        }
    }

    let db = rusqlite::Connection::open(&real_path).expect("Connection to the db failed");
    std::fs::set_permissions(&real_path, std::fs::Permissions::from_mode(0o600)).unwrap();
    let _ = create_table(&db);
    db
}

fn db_cleanup(filename: &str, db: &rusqlite::Connection) {
    // Cleanup the connection with the database. Checks if the
    // database is where it should or in a temporary location. If it
    // is the temporary location let's copy it back to where it should
    // be.
    //
    // Input:
    //     filename: where the database should be
    //     db: connection to the database
    let db_path = db
        .path()
        .expect("Seems we had no database to work with....")
        .to_str()
        .expect("Failed to convert path to str");
    if db_path != filename {
        std::fs::copy(&db_path, filename)
            .expect("Failed to copy the db file to the final location");
        std::fs::remove_file(db_path).expect("Failed to delete temporary file on the shm device");
    }
}

#[pyfunction(mol2_list, filename, append = "false")]
pub fn write_mol2(mol2_list: Vec<Mol2>, filename: &str, append: bool) {
    // Write a vector of mol2 structures to a single mol2 file
    // Input:
    //     mol2_list: vector with structures
    //     filename: desired path for the final mol2 file
    //
    // TODO: At some point we might need to add some buffering in case
    // the list is too large...
    //
    // TODO: At some point we probably want an 'append' option.
    let mut text = String::new();
    for entry in &mol2_list {
        text.push_str(&entry.as_string());
    }
    write_string(&text, filename, append);
}

pub fn db_insert(mol2_list: Vec<Mol2>, filename: &str, compression: i32, shm: bool) {
    // Insert vector of structures into a database. Append if the database exists.
    // Input:
    //     mol2_list: vector of structures
    //     filename: path to the database
    //     compression: level of zstd compression. NOTE: 0 means no compression and not default level as in zstd library
    //     shm: should be try and use a database out from a temporary location
    let db = get_db(filename, shm);
    let _ = create_table(&db);
    let mut insert_cmd: String = String::new();
    insert_cmd.push_str("INSERT INTO structures (mol_name, num_atoms, num_bonds, num_subst, num_feat, num_sets, mol_type, charge_type, status_bits, mol_comment, atom, bond, substructure, compression, desc) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)");
    // Handle compression levels
    let mut compression_level = compression;
    if compression_level > 9 {
        compression_level = 9;
    }
    let mut statement = db
        .prepare(&insert_cmd)
        .expect("Failed to prepare an sql statement");
    for entry in mol2_list.iter() {
        let mut atom = bincode::serialize(&entry.atom).expect("Failed to serialize into binary");
        let mut bond = bincode::serialize(&entry.bond).expect("Failed to serialize into binary");
        let mut subs =
            bincode::serialize(&entry.substructure).expect("Failed to serialize into binary");
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
        statement
            .execute(rusqlite::params![
                entry.molecule.as_ref().unwrap().mol_name,
                entry.molecule.as_ref().unwrap().num_atoms,
                entry.molecule.as_ref().unwrap().num_bonds,
                entry.molecule.as_ref().unwrap().num_subst,
                entry.molecule.as_ref().unwrap().num_feat,
                entry.molecule.as_ref().unwrap().num_sets,
                entry.molecule.as_ref().unwrap().mol_type,
                entry.molecule.as_ref().unwrap().charge_type,
                entry.molecule.as_ref().unwrap().status_bits,
                entry.molecule.as_ref().unwrap().mol_comment,
                atom,
                bond,
                subs,
                compression_level,
                entry.desc,
            ])
            .expect("Failed to insert data to db");
    }

    db_cleanup(filename, &db);
}

#[pyfunction(mol2_list, filename, compression = "3", shm = "true")]
#[pyo3(name = "db_insert")]
fn py_db_insert(mol2_list: Vec<Mol2>, filename: &str, compression: i32, shm: bool) {
    db_insert(mol2_list, filename, compression, shm)
}

pub fn read_db_all(filename: &str, shm: bool, desc: &str, comment: &str) -> Vec<Mol2> {
    // Read all structures from a database and return as a vector
    // Input:
    //     filename: path to the database
    //     shm: should we try and use the database out of a temporary location?
    let db = get_db(filename, shm);
    let mut stmt = db.prepare("SELECT mol_name, num_atoms, num_bonds, num_subst, num_feat, num_sets, mol_type, charge_type, status_bits, mol_comment, atom, bond, substructure, compression, desc FROM structures").expect("Failed to fetch from the database");
    let structure_iter = stmt
        .query_map([], |row| {
            let compression: i32 = row.get(13).unwrap();
            let mut atom: Vec<u8> = row.get(10).unwrap();
            let mut bond: Vec<u8> = row.get(11).unwrap();
            let mut subs: Vec<u8> = row.get(12).unwrap();
            if compression > 0 {
                atom = zstd::block::Decompressor::new()
                    .decompress(&atom, DECOMPRESSOR_BUFFER)
                    .expect("Failed to decompress");
                bond = zstd::block::Decompressor::new()
                    .decompress(&bond, DECOMPRESSOR_BUFFER)
                    .expect("Failed to decompress");
                subs = zstd::block::Decompressor::new()
                    .decompress(&subs, DECOMPRESSOR_BUFFER)
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
                desc: row.get(14).unwrap(),
            })
        })
        .expect("Failed to fetch exact numbers from db");
    let mut mol2_list: Vec<Mol2> = Vec::new();
    for structure in structure_iter {
        mol2_list.push(structure.expect("Failed to get structure after successful extraction...?"));
    }
    if !desc.is_empty() {
        mol2_list.retain(|mol2| mol2.desc.as_ref().unwrap_or(&String::new()).contains(desc));
    }
    if !comment.is_empty() {
        mol2_list.retain(|mol2| {
            mol2.molecule
                .as_ref()
                .unwrap_or(&Molecule::new())
                .mol_comment
                .as_ref()
                .unwrap_or(&String::new())
                .contains(desc)
        });
    }

    mol2_list
}

#[pyfunction(filename, shm = "false", desc = "\"\"", comment = "\"\"")]
#[pyo3(name = "read_db_all")]
fn py_read_db_all(filename: &str, shm: bool, desc: &str, comment: &str) -> Vec<Mol2> {
    read_db_all(filename, shm, desc, comment)
}

#[pyfunction(filename, shm = "false", desc = "\"\"", comment = "\"\"")]
fn read_db_all_serialized(
    filename: &str,
    shm: bool,
    desc: &str,
    comment: &str,
) -> PyResult<Vec<PyObject>> {
    // Read all structures from a database and return as a vector, but
    // keep structures in a serialized python form rather than binary.
    // Input:
    //     filename: path to the database
    //     shm: should we try and use the database out of a temporary location?
    let mol2_list = read_db_all(filename, shm, desc, comment);
    let mut result: Vec<PyObject> = Vec::new();
    for entry in &mol2_list {
        result.push(
            entry
                .serialized()
                .expect("Failed to serialize mol2 entry..."),
        );
    }
    Ok(result)
}

#[pyfunction(filename, shm = "false")]
pub fn desc_list(filename: &str, shm: bool) -> Vec<String> {
    // Read all structures from a database and return as a vector
    // Input:
    //     filename: path to the database
    //     shm: should we try and use the database out of a temporary location?
    let db = get_db(filename, shm);
    let mut stmt = db
        .prepare("SELECT desc FROM structures")
        .expect("Failed to fetch from the database");
    let desc_iter = stmt
        .query_map([], |row| Ok(row.get(0).unwrap()))
        .expect("Failed to fetch exact numbers from db");
    let mut desc_list: Vec<String> = Vec::new();
    for desc in desc_iter {
        desc_list.push(desc.expect("Failed to get desc after successful extraction...?"));
    }

    desc_list.sort();
    desc_list.dedup();

    desc_list
}

pub fn read_file_to_db(
    filename: &str,
    db_name: &str,
    compression: i32,
    shm: bool,
    desc: &str,
    comment: &str,
) {
    // Convenience function. Read structures from a mol2 file and write directly to the database
    // Input:
    //     filename: path to the mol2 file
    //     db_name: path to the database
    //     compression: compression level
    //     shm: should we use the database out of a temporary location
    let content = read_file(filename, desc, comment);
    let _ = db_insert(content, db_name, compression, shm);
}

#[pyfunction(
    filename,
    db_name,
    compression = "3",
    shm = "true",
    desc = "\"\"",
    comment = "\"\""
)]
#[pyo3(name = "read_file_to_db")]
fn py_read_file_to_db(
    filename: &str,
    db_name: &str,
    compression: i32,
    shm: bool,
    desc: &str,
    comment: &str,
) {
    read_file_to_db(filename, db_name, compression, shm, desc, comment)
}

pub fn read_file_to_db_batch(
    filenames: Vec<&str>,
    db_name: &str,
    compression: i32,
    shm: bool,
    desc: &str,
    filename_desc: bool,
    comment: &str,
) {
    // Convenience function. Read structures from a set of files directly into the database
    // Input:
    //     filenames: vector of paths to mol2 files
    //     db_name: path to the database
    //     compression: compression level
    //     shm: should we use the database out of a temporary location
    for filename in &filenames {
        let mut description: String = desc.to_owned();
        if filename_desc {
            if !desc.is_empty() {
                description.push_str("; ");
            }
            description.push_str(
                std::path::Path::new(filename)
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new(filename))
                    .to_str()
                    .unwrap_or(filename),
            );
        }
        let content = read_file(filename, &description, comment);
        let _ = db_insert(content, db_name, compression, shm);
    }
}

#[pyfunction(
    filenames,
    db_name,
    compression = "3",
    shm = "true",
    desc = "\"\"",
    filename_desc = "false",
    comment = "\"\""
)]
#[pyo3(name = "read_file_to_db_batch")]
fn py_read_file_to_db_batch(
    filenames: Vec<&str>,
    db_name: &str,
    compression: i32,
    shm: bool,
    desc: &str,
    filename_desc: bool,
    comment: &str,
) {
    read_file_to_db_batch(
        filenames,
        db_name,
        compression,
        shm,
        desc,
        filename_desc,
        comment,
    )
}

pub fn read_file(filename: &str, desc: &str, comment: &str) -> Vec<Mol2> {
    // Read a mol2 file and return a vector of structures
    // Input:
    //     filename: path to a mol2 file
    let file = File::open(filename).expect("Failed to open the input file");
    let reader = BufReader::with_capacity(100 * 1024 * 1024, file);
    let mut section_name: String = String::new();
    let mut section_index: usize = 0;
    let mut mol2: Vec<Mol2> = Vec::new();
    let mut entry = Mol2::new(desc);
    for (index, line) in reader.lines().enumerate() {
        let line =
            line.expect(&format!("Failed to read {} line from the input file", index + 1)[..]);

        let mut section_start = false;
        if line.len() > 11 {
            section_start = &line[0..9] == "@<TRIPOS>";
        }
        if section_start {
            section_name = (&line[9..]).to_owned();
            // make sure to not use any extra characters...
            section_name = section_name.split_whitespace().next().unwrap().to_owned();
            section_index = index;
            if section_name == "MOLECULE" && entry.molecule.is_some() {
                entry.add_comment(comment);
                mol2.push(entry);
                entry = Mol2::new(desc);
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
    entry.add_comment(comment);
    mol2.push(entry); // if we are just at the end of the file

    mol2
}

#[pyfunction(filename, desc = "\"\"", comment = "\"\"")]
#[pyo3(name = "read_file")]
fn py_read_file(filename: &str, desc: &str, comment: &str) -> Vec<Mol2> {
    read_file(filename, desc, comment)
}

#[pyfunction(filename, desc = "\"\"", comment = "\"\"")]
fn read_file_serialized(filename: &str, desc: &str, comment: &str) -> PyResult<Vec<PyObject>> {
    // Read a mol2 file and return a vector of structures, but
    // serialized python structures rather than a binary form.
    // Input:
    //     filename: path to a mol2 file
    let mol2_list = read_file(filename, desc, comment);
    let mut result: Vec<PyObject> = Vec::new();
    for entry in &mol2_list {
        result.push(
            entry
                .serialized()
                .expect("Failed to serialize mol2 entry..."),
        );
    }
    Ok(result)
}

#[pymodule]
fn serde_mol2(_py: Python, m: &PyModule) -> PyResult<()> {
    // Define a python module.
    m.add_class::<Molecule>()?;
    m.add_class::<Atom>()?;
    m.add_class::<Bond>()?;
    m.add_class::<Substructure>()?;
    m.add_class::<Mol2>()?;
    m.add_wrapped(wrap_pyfunction!(py_read_file))?;
    m.add_wrapped(wrap_pyfunction!(read_file_serialized))?;
    m.add_wrapped(wrap_pyfunction!(py_db_insert))?;
    m.add_wrapped(wrap_pyfunction!(py_read_db_all))?;
    m.add_wrapped(wrap_pyfunction!(read_db_all_serialized))?;
    m.add_wrapped(wrap_pyfunction!(py_read_file_to_db))?;
    m.add_wrapped(wrap_pyfunction!(py_read_file_to_db_batch))?;
    m.add_wrapped(wrap_pyfunction!(write_mol2))?;
    m.add_wrapped(wrap_pyfunction!(desc_list))?;

    Ok(())
}
