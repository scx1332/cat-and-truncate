# cat-once

Cat and truncate file at the same time, by recopying it to itself.


# Sample usage

# Unpack files from zst tar to output directory
cat-once --file test2.tar.zst --truncate 33 | tar -I zstd -xf - -C output


cat-once --dry-run --file heimdall-mainnet-snapshot-bulk-2024-02-01.tar.zst -c 1010765k | tar -I zstd -xf - -C output