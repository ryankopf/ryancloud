
if [ "$(id -u)" -ne 0 ]; then
  echo "Please run as root"
  exit
fi
chmod +x next.sh build.sh
service ryancloud stop
cp ./target/release/ryancloud /usr/local/bin/ryancloud
chmod 755 /usr/local/bin/ryancloud
# Grant capability to bind to port 80 without root
setcap 'cap_net_bind_service=+ep' /usr/local/bin/ryancloud
service ryancloud restart
