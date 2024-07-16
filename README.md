# cat-once

Cat and truncate file at the same time, by recopying it to itself.


# Sample usage

# Unpack files from zst tar to output directory
cat-once --file test2.tar.zst --truncate 33 | tar -I zstd -xf - -C output