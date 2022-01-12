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

use clap::{App, Arg};

fn main() {
    let args = App::new("serde-mol2")
        .version("0.2.0")
        .author(
            "CSC - IT Center for Science Ltd. (Jaroslaw Kalinowski <jaroslaw.kalinowski@csc.fi>)",
        )
        .arg(
            Arg::new("input_file")
                .short('i')
                .long("input")
                .value_name("INPUT_FILE")
                .help("Input mol2 file")
                .takes_value(true)
                .multiple_values(true),
        )
        .arg(
            Arg::new("output_file")
                .short('o')
                .long("output")
                .value_name("OUTPUT_FILE")
                .help("Output mol2 file")
                .takes_value(true),
        )
        .arg(
            Arg::new("sqlite")
                .short('s')
                .long("sqlite")
                .value_name("SQLITE_FILE")
                .help("Sqlite database file")
                .takes_value(true),
        )
        .arg(
            Arg::new("append")
                .short('a')
                .long("append")
                .help("Append to mol2 files when writing rather than truncate"),
        )
        .arg(
            Arg::new("no_shm")
                .long("no-shm")
                .help("Do not try using shm device when writing to databases"),
        )
        .arg(
            Arg::new("desc")
                .long("desc")
                .value_name("DESC")
                .help("Description to add/filter to/by entries when writing to the database")
                .takes_value(true),
        )
        .arg(
            Arg::new("comment")
                .long("comment")
                .value_name("COMMENT")
                .help("Comment to add/filter to/by the molecule comment field")
                .takes_value(true),
        )
        .arg(
            Arg::new("compression")
                .short('c')
                .long("compression")
                .value_name("COMPRESSION")
                .default_value("3")
                .help("Level of compression for BLOB data, 0 means no compression")
                .takes_value(true),
        )
        .arg(
            Arg::new("filename_desc").long("filename-desc").help(
                "Add filename to the desc field when adding a batch of files to the database",
            ),
        )
        .arg(
            Arg::new("list_desc")
                .long("list-desc")
                .help("List available row descriptions present in the database"),
        )
        .get_matches();

    // different variants I guess... might be a long tree of if's. Hopefully later will make it nicer

    // simple reading input files into the database
    if args.is_present("input_file") && args.is_present("sqlite") {
        let input_files = args.values_of("input_file");
        let input_files: Vec<&str> = input_files.expect("No input files after all").collect();
        if input_files.len() > 1 {
            serde_mol2::read_file_to_db_batch(
                input_files,
                args.value_of("sqlite").expect(
                    "There seem to be input files given but no sqlite file to operate with",
                ),
                args.value_of("compression")
                    .expect("Missing compression level...")
                    .parse::<i32>()
                    .expect("Failed to parse compression level"),
                !args.is_present("no_shm"),
                args.value_of("desc").unwrap_or(""),
                args.is_present("filename_desc"),
                args.value_of("comment").unwrap_or(""),
            );
        } else {
            serde_mol2::read_file_to_db(
                input_files[0],
                args.value_of("sqlite").expect(
                    "There seem to be input files given but no sqlite file to operate with",
                ),
                args.value_of("compression")
                    .expect("Missing compression level...")
                    .parse::<i32>()
                    .expect("Failed to parse compression level"),
                !args.is_present("no_shm"),
                args.value_of("desc").unwrap_or(""),
                args.value_of("comment").unwrap_or(""),
            );
        }
    }
    // simple reading database into mol2 file
    if args.is_present("output_file") && args.is_present("sqlite") {
        let mol2_list = serde_mol2::read_db_all(
            args.value_of("sqlite")
                .expect("Missing sqlite db filename after all..."),
            !args.is_present("no_shm"),
            args.value_of("desc").unwrap_or(""),
            args.value_of("comment").unwrap_or(""),
        );
        serde_mol2::write_mol2(
            mol2_list,
            args.value_of("output_file")
                .expect("Missing output file argument after all"),
            args.is_present("append"),
        );
    }

    // At the end list available desc fields if requested
    if args.is_present("list_desc") && args.is_present("sqlite") {
        let desc_list = serde_mol2::desc_list(
            args.value_of("sqlite")
                .expect("Missing sqlite db filename after all..."),
            !args.is_present("no_shm"),
        );
        for desc in desc_list {
            println!("{}", desc);
        }
    }
}
