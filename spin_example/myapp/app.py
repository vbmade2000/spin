from spin_sdk.http import IncomingHandler, Request, Response
import time

class IncomingHandler(IncomingHandler):
    def handle_request(self, request: Request) -> Response:
        # time.sleep(1000)
        return Response(
            200,
            {"content-type": "text/plain"},
            bytes("Hello from myapp!", "utf-8")
        )
