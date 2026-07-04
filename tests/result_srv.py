import threading
from http.server import BaseHTTPRequestHandler, HTTPServer
success = False
test_nbr = 0
number_of_success_expected = 0
success_counter = 0
multiple_objectives = False
done_objectives = []

def set_test(nbr, counter, multiple_obj: bool = False):
    global success, test_nbr, number_of_success_expected, success_counter, multiple_objectives
    success = False
    test_nbr = nbr
    multiple_objectives = multiple_obj
    number_of_success_expected = counter
    success_counter = 0

def is_success():
    global success
    return success

class ResultServer(BaseHTTPRequestHandler):
    @staticmethod
    def threaded_server(port):
        result_server = HTTPServer(('', port), ResultServer)
        threading.Thread(target=result_server.serve_forever, args=(), daemon=True).start()
        return result_server

    def __init__(self, *args, **kwargs):
        global test_nbr, success, number_of_success_expected
        super().__init__(*args, **kwargs)


    def do_GET(self):
        global success, test_nbr, number_of_success_expected, success_counter
        self.send_response(200)
        self.send_header('Content-type', 'text/html')
        if self.path.strip() == f"/r/{test_nbr}/ok":
            success_counter += 1
            if success_counter >= number_of_success_expected:
                print(f"Success: {success_counter}/{number_of_success_expected}")
                success = True
        elif multiple_objectives and self.path.strip().startswith(f"/r/{test_nbr}/"):
            print(f"Fail: {success_counter}/{number_of_success_expected}")
        if self.path.strip().startswith(f"/r/{test_nbr}/ok/") and multiple_objectives:
            server_nbr = self.path.strip().split("/")[-1]
            if server_nbr not in done_objectives:
                done_objectives.append(server_nbr)
                success_counter += 1
                if success_counter >= number_of_success_expected:
                    print(f"Success: {success_counter}/{number_of_success_expected}")
                    success = True
        self.wfile.write(bytes(f"Success: {success_counter}/{number_of_success_expected}", "utf-8"))
        self.end_headers()
