import socket
import threading
from typing_extensions import KeysView
import sys

sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.connect(("139.162.200.195", 8080))

ON = True

def recv_method(sock):
    while ON:
        try:
            data = sock.recv(1024).decode()
            print(data)
        except BrokenPipeError as e:
            print(f"Error {e}")
            ON = False
        except:
            pass

def send_method(sock):
    while ON:
        try:
            data = input(">").encode()
            sock.send_all(data)
        except BrokenPipeError as e:
            print(f"Error {e}")
            ON = False
        except KeyboardInterrupt:
            print("KI. Disconnecting.")
            ON = False
        except:
            pass

recv_thread = threading.Thread(target=recv_method, args=(sock,))
send_thread = threading.Thread(target=send_method, args=(sock,))

recv_thread.start()
send_thread.start()

recv_thread.join()
send_thread.join()

sock.close()
sys.exit()
