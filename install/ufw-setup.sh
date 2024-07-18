echo "Setting up UFW firewall application profile"

cp ./ufw.ini /etc/ufw/applications.d/sailor
chown root /etc/ufw/applications.d/sailor
chmod 0600 /etc/ufw/applications.d/sailor
ufw allow Sailor

exit
