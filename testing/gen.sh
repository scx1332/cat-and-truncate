set -x

mkdir tmp
sudo mount -o size=2G -t tmpfs none tmp
cd tmp
cargo run --release -- --file test.file --test-create-ascii-file-size=1000000000
tar -cf - test.file | zstd -c > test.tar.zstd
cp test.tar.zstd ../test.tar.zstd
cd ..
sudo umount tmp
rm -r tmp

