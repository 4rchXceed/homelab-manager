import socket

from pyfiglet import Figlet

socket_path = "/tmp/homelabmanager.sock"

client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)

client.connect(socket_path)

stop = False
f = Figlet(font="slant")
print(f.renderText("Homelab Manager CLI"))
print("exit to exit")
while not stop:
    message = input("admin@hlabmgr: ")
    client.sendall(message.encode())
    command_ended = False
    while not command_ended:
        response = client.recv(1024)
        if response.decode().strip().endswith(
            "OK"
        ) or response.decode().strip().endswith("ERROR"):
            command_ended = True
            if response.decode().strip().endswith("OK"):
                print(f"{response.decode().strip().removesuffix('OK')}\n")
                print("[+] ", end="")
            if response.decode().strip().endswith("ERROR"):
                print(f"{response.decode().strip().removesuffix('ERROR')}\n")
                print("[-] ", end="")
        else:
            if response.decode().strip().endswith("::INPUT"):
                print(f"{response.decode().strip().removesuffix('::INPUT')}", end="")
                answer = input("")
                client.sendall(answer.encode())
            else:
                print(f"{response.decode()}\n")
    if message.lower() == "exit":
        stop = True
