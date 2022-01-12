#!/usr/bin/env python3
import serde_mol2
import argparse

def main(args):
    '''main...'''

    if args.input and args.sqlite:
        serde_mol2.read_file_to_db_batch(args.input, args.sqlite, shm = not args.no_shm, desc = args.desc, comment = args.comment, compression = int(args.compress))

    if args.output and args.sqlite:
        m = serde_mol2.read_db_all(args.sqlite, desc = args.desc, comment = args.comment)
        serde_mol2.write_mol2(m, args.output)

    if args.list_desc and args.sqlite:
        for desc in serde_mol2.desc_list(args.sqlite):
            print(desc)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(usage=__doc__)
    parser.add_argument(
        '-i',
        '--input',
        nargs='+',
        help="Mol2 files to read"
    )
    parser.add_argument(
        '-o',
        '--output',
        help="mol2 files to write to"
    )
    parser.add_argument(
        '-s',
        '--sqlite',
        help="Sqlite db to work with"
    )
    parser.add_argument(
        '-c',
        '--compress',
        default='3',
        help="Sqlite database to write to"
    )
    parser.add_argument(
        '--no-shm',
        action="store_true",
        help="Do not use shm device for temporary storage"
    )
    parser.add_argument(
        '--list-desc',
        action="store_true",
        help="Do not use shm device for temporary storage"
    )
    parser.add_argument(
        '--desc',
        default='',
        help="Description to add or filter by"
    )
    parser.add_argument(
        '--comment',
        default='',
        help="Comment to add or filter by"
    )

    main(parser.parse_args())
