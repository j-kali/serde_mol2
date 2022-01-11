#!/usr/bin/env python3
import serde_mol2
import argparse

def main(args):
    '''main...'''

    if args.input:
        if not args.read_db:
            if args.just_read:
                for input_file in args.input:
                    m = serde_mol2.read_file(input_file)
            else:
                if not args.write_mol2:
                    serde_mol2.read_file_to_db_batch(args.input, args.output, shm = not args.no_shm)
                else:
                    m = []
                    for input_file in args.input:
                        m += serde_mol2.read_file(input_file)
                    serde_mol2.write_mol2(m, args.output)
        else:
            # we are reading db and outputting mol2
            m = []
            for input_file in args.input:
                m += serde_mol2.read_db_all(input_file)
            serde_mol2.write_mol2(m, args.output)
    else:
        print("No input file given...")

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
        default='db.sqlite',
        help="Sqlite database to write to"
    )
    parser.add_argument(
        '-c',
        '--compress',
        default='3',
        help="Sqlite database to write to"
    )
    parser.add_argument(
        '--just-read',
        action="store_true",
        help="Just read the files"
    )
    parser.add_argument(
        '--no-shm',
        action="store_true",
        help="Do not use shm device for temporary storage"
    )
    parser.add_argument(
        '--write-mol2',
        action="store_true",
        help="Do not use shm device for temporary storage"
    )
    parser.add_argument(
        '--read-db',
        action="store_true",
        help="Do not use shm device for temporary storage"
    )

    main(parser.parse_args())
