set -x

mkdir tmp
sudo mount -o size=1600M -t tmpfs none tmp
cp test.tar.zst tmp/test.tar.zst
(cd tmp && tar -xf test.tar.zst)

sudo umount tmp
rm -r tmp

