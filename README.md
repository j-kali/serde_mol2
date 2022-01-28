# serde_mol2
Python/Rust module for mol2 format (de)serialization

## Installation

Install from [PyPi](https://pypi.org/project/serde-mol2/) (required python >= 3.8):

    pip install serde-mol2

After that:

    -> python3
    Python 3.9.5 (default, Jun  4 2021, 12:28:51)
    [GCC 7.5.0] :: Anaconda, Inc. on linux
    Type "help", "copyright", "credits" or "license" for more information.
    >>> import serde_mol2
    >>> m = serde_mol2.read_file('example.mol2')
    >>> m
    [<builtins.Mol2 object at 0x7f6da9ebcae0>]

Or using a binary:

    -> serde-mol2 -h
    serde-mol2 0.2.2
    CSC - IT Center for Science Ltd. (Jaroslaw Kalinowski <jaroslaw.kalinowski@csc.fi>)

    USAGE:
        serde-mol2 [OPTIONS]

    OPTIONS:
        -a, --append                       Append to mol2 files when writing rather than truncate
        -c, --compression <COMPRESSION>    Level of compression for BLOB data, 0 means no compression
                                           [default: 3]
            --comment <COMMENT>            Comment to add/filter to/by the molecule comment field
            --desc <DESC>                  Description to add/filter to/by entries when writing to the
                                           database
            --filename-desc                Add filename to the desc field when adding a batch of files
                                           to the database
        -h, --help                         Print help information
        -i, --input <INPUT_FILE>...        Input mol2 file
            --limit <LIMIT>                Limit the number of structures retrieved from the database.
                                           Zero means no limit. [default: 0]
            --list-desc                    List available row descriptions present in the database
            --no-shm                       Do not try using shm device when writing to databases
        -o, --output <OUTPUT_FILE>         Output mol2 file
            --offset <OFFSET>              Offset when limiting the number of structures retrieved from
                                           the database. Zero means no offset. [default: 0]
        -s, --sqlite <SQLITE_FILE>         Sqlite database file
        -V, --version                      Print version information

## Usage a.k.a. quick function reference

### class Mol2

- Mol2.to_json()

  Return a `JSON` string for a `Mol2` object.

- Mol2.as_string()

  Return a `mol2` string for a `Mol2` object.

- Mol2.write_mol2( _filename_, _append=False_ )

  Write `Mol2` object to a `mol2` file.

- Mol2.serialized()

  Return a `Mol2` object in a python serialized form.

### Functions

- write_mol2( _list_, _filename_, _append=False_ )

  _list_  is a list of `Mol2` objects. Functions writes all structures in the list into a `mol2` file named _filename_.

- db_insert( _list_, _filename_, _compression=3_, _shm=True_ )

  Insert vector of structures into a database. Append if the database exists.

  Input:
  * _list_: vector of structures
  * _filename_: path to the database
  * _compression_: compression level
  * _shm_: should be try and use a database out from a temporary location?

- read_db_all( _filename_, _shm=False_, _desc=None_, _comment=None_, _limit=0_, _offset=0_ )

  Read all structures from a database and return as a vector

  Input:
  * _filename_: path to the database
  * _shm_: should we try and use the database out of a temporary location?
  * _desc_: return only entries containing _desc_ in the _desc_ field
  * _comment_: return only entries containing _comment_ in the molecule comment
  * _limit_: Limit the number of structures retrieved from the database and zero means no limit
  * __offset_: Offset when limiting the number of structures retrieved from the database and zero means no offset

- read_db_all_serialized( _filename_, _shm=True_, _desc=None_, _comment=None_, _limit=0_, _offset=0_ )

  Read all structures from a database and return as a vector, but
  keep structures in a serialized python form rather than binary.

  Input:
  * _filename_: path to the database
  * _shm_: should we try and use the database out of a temporary location?
  * _desc_: return only entries containing _desc_ in the _desc_ field
  * _comment_: return only entries containing _comment_ in the molecule comment
  * _limit_: Limit the number of structures retrieved from the database and zero means no limit
  * __offset_: Offset when limiting the number of structures retrieved from the database and zero means no offset

- read_file_to_db( _filename_, _db-filename_, _compression=3_, _shm=True_ , _desc=None_, _comment=None_ )

  Convenience function. Read structures from a mol2 file and write directly to the database.

  Input:
  * _filename_: path to the mol2 file
  * _db-filename_: path to the database
  * _compression_: compression level
  * _shm_: should we use the database out of a temporary location?
  * _desc_: add this description to structures read
  * _comment_: add this comment to the molecule comment field

- read_file_to_db_batch( _filenames_, _db-filename_, _compression=3_, _shm=True_, _desc=None_, _comment=None_ )

  Convenience function. Read structures from a set of files directly into the database.

  Input:
  * _filenames_: vector of paths to mol2 files
  * _db-filename_: path to the database
  * _compression_: compression level
  * _shm_: should we use the database out of a temporary location?
  * _desc_: add this description to structures read
  * _comment_: add this comment to the molecule comment field

- read_file( _filename_, _desc=None_, _comment=None_ )

  Read a mol2 file and return a vector of structures

  Input:
  * _filename_: path to the mol2 file
  * _desc_: add this description to structures read
  * _comment_: add this comment to the molecule comment field

- read_file_serialized( _filename_, _desc=None_, _comment=None_ )

  Read a mol2 file and return a vector of structures, but
  serialized python structures rather than a binary form.

  Input:
  * _filename_: path to the mol2 file
  * _desc_: add this description to structures read
  * _comment_: add this comment to the molecule comment field

- desc_list( _filename_, _shm=False_ )

  List unique entry descriptions found in a database.

  Input:
  * _filename_: path to a database
  * _shm_: should we use the database out of a temporary location?

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
