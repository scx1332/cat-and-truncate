set -x

mkdir tmp
sudo mount -o size=1510M -t tmpfs none tmp
cp test.tar.zstd tmp/test.tar.zstd
(cd tmp && cargo run --release -- --truncate=30 -s 1 --file test.tar.zstd | tar -I zstd -xf - -C .)

sudo umount tmp
rm -r tmp

