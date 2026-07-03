# Create 2 virtual network interfaces
if ! command -v ifconfig >/dev/null 2>&1; then
    echo "ifconfig command not found. Please install net-tools package."
    exit 1
fi

sudo modprobe dummy
sudo ip link add veth0 type dummy
sudo ip link add veth1 type dummy
sudo ifconfig veth0 hw ether C8:D7:4A:4E:47:50
sudo ifconfig veth1 hw ether C8:D7:4A:4E:47:51
sudo ip addr add 192.168.239.10/24 brd + dev veth0 label veth0:0
sudo ip addr add 192.168.239.11/24 brd + dev veth1 label veth1:0
sudo ip link set dev veth0 up
sudo ip link set dev veth1 up
