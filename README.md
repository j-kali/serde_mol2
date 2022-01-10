# serde_mol2
Python/Rust module for mol2 format (de)serialization

## Installation

For now only local use is supported.

    make develop

Should be enough. For compilation you're going to need a working `cargo`+`rustup`. After that:

    -> python3
    Python 3.9.5 (default, Jun  4 2021, 12:28:51)
    [GCC 7.5.0] :: Anaconda, Inc. on linux
    Type "help", "copyright", "credits" or "license" for more information.
    >>> import serde_mol2
    >>> m = serde_mol2.read_file('example.mol2')
    >>> m
    [<builtins.Mol2 object at 0x7f6da9ebcae0>]

## Usage a.k.a. quick function reference

### class Mol2

- Mol2.to_json()

  Return a `JSON` string for a `Mol2` object.

- Mol2.as_string()

  Return a `mol2` string for a `Mol2` object.

- Mol2.write_mol2( _filename_ )

  Write `Mol2` object to a `mol2` file.

- Mol2.serialized()

  Return a `Mol2` object in a python serialized form.

### Functions

- write_mol2( _list_, _filename_ )

  _list_  is a list of `Mol2` objects. Functions writes all structures in the list into a `mol2` file named _filename_.

- db_insert( _list_, _filename_, _compression=3_, _shm=True_ )

  Insert vector of structures into a database. Append if the database exists.

  Input:
  * _list_: vector of structures
  * _filename_: path to the database
  * _compression_: compression level
  * _shm_: should be try and use a database out from a temporary location?

- read_db_all( _filename_, _shm=False_ )

  Read all structures from a database and return as a vector

  Input:
  * _filename_: path to the database
  * _shm_: should we try and use the database out of a temporary location?

- read_db_all_serialized( _filename_, _shm=True_ )

  Read all structures from a database and return as a vector, but
  keep structures in a serialized python form rather than binary.

  Input:
  * _filename_: path to the database
  * _shm_: should we try and use the database out of a temporary location?

- read_file_to_db( _filename_, _db-filename_, _compression=3_, _shm=True_ , _desc=None_ )

  Convenience function. Read structures from a mol2 file and write directly to the database.

  Input:
  * _filename_: path to the mol2 file
  * _db-filename_: path to the database
  * _compression_: compression level
  * _shm_: should we use the database out of a temporary location?
  * _desc_: add this description to structures read

- read_file_to_db_batch( _filenames_, _db-filename_, _compression=3_, _shm=True_ _desc=None_ )

  Convenience function. Read structures from a set of files directly into the database.

  Input:
  * _filenames_: vector of paths to mol2 files
  * _db-filename_: path to the database
  * _compression_: compression level
  * _shm_: should we use the database out of a temporary location?
  * _desc_: add this description to structures read

- read_file( _filename_, _desc=None_ )

  Read a mol2 file and return a vector of structures

  Input:
  * _filename_: path to the mol2 file
  * _desc_: add this description to structures read

- read_file_serialized( _filename_, _desc=None_ )

  Read a mol2 file and return a vector of structures, but
  serialized python structures rather than a binary form.

  Input:
  * _filename_: path to the mol2 file
  * _desc_: add this description to structures read

### Notes

#### Compression

Compression applies to sections other than `MOLECULE`. Those sections are stored in the database in a binary form (`BLOB`) as those sections contain multiple rows. Since it is not human readable it makes sense to apply at least some compression. The algorithm of choice currently is [`zstd`](https://github.com/facebook/zstd). Default level of compression here is 3. **However**, by default, for `zstd` compression 0 means default level of compression, but in this module compression level 0 means no compression.

At the time of writing the overhead that comes from (de)compressing the data is negligible compared to IO/CPU cost of rw and parsing.

#### SHM

When writing to the database we are writing just one row at a time. On shared filesystems writing like that is very slow. When using `shm` functionality the module tries to copy the database to `/dev/shm` and use it there, essentially performing all operations in-memory. However, this means that file in the original location is essentially not usable by other processes as it will be overwritten at the end.

Another problem with doing things in `/dev/shm` is that if the database is too big, we can run out of space. So make sure your database fits into memory available.

In the future there will be an option to choose a different `TMPDIR` than `/dev/shm`, for example one that points to a fast `NVMe` storage.

By default `shm` is used only when writing to the database, as reading seems to not be affected so much.

#### Limitations

The biggest limitation at the moment is that only the following sections are read:

* MOLECULE
* ATOM
* BOND
* SUBSTRUCTURE

All other sections are currently just dropped silently.
