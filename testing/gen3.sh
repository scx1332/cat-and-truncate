set -x

mkdir tmp
sudo mount -o size=1510M -t tmpfs none tmp
cp test.tar.zst tmp/test.tar.zst
(cd tmp && cargo run --release -- --truncate=30 -s 1 --file test.tar.zst | tar -I zstd -xf - -C .)

sudo umount tmp
rm -r tmp

