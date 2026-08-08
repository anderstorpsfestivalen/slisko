import asyncio
import websockets
from apa102_pi.driver import apa102
strip = apa102.APA102(num_led=132, mosi=10, sclk=11, order='rbg')
strip.set_global_brightness(31)
strip.clear_strip()


async def data(websocket, path):
        async for recv_data in websocket:
            bytes = bytearray(recv_data)

            for i, (r, g, b) in enumerate(zip(*[iter(bytes)]*3)):
                strip.set_pixel(i, r, g, b)

            strip.show()

start_server = websockets.serve(data, "0.0.0.0", 8765)

asyncio.get_event_loop().run_until_complete(start_server)
asyncio.get_event_loop().run_forever()

