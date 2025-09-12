if [ "$EUID" -ne 0 ]; then
  echo "Please run as root"
  exit
fi
cp ./target/release/ryancloud /usr/local/bin/ryancloud
chmod 755 /usr/local/bin/ryancloud
service ryancloud restart