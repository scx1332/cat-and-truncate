
file_size=1000
chunk_size=1

cargo run --release -- --file test.file --test-create-ascii-file-size=$file_size
sha1sum test.file > test-in.file.sha1
cargo run --release -- --file test.file --chunk-size $chunk_size > test-out.file
# the file should disappear
mv test-out.file test.file
sha1sum test.file > test-out.file.sha1
