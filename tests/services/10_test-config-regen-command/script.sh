echo "NO" > file
curl -s http://192.168.239.10:5001/r/10/ok/1

while true; do
  if cat file | grep -q "OK"; then
    echo "OK"
    curl -s http://192.168.239.10:5001/r/10/ok/2
    break
  fi
  sleep 1
done
