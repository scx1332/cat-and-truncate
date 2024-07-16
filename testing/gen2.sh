set -x

mkdir tmp
sudo mount -o size=1600M -t tmpfs none tmp
cp test.tar.zstd tmp/test.tar.zstd
(cd tmp && tar -xf test.tar.zstd)

sudo umount tmp
rm -r tmp

