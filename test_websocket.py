
import websocket
import json
import rel

addr = "ws://127.0.0.1:12345/join_game"


def on_message(ws, message):
    print(message)


def on_error(ws, error):
    print(error)


def on_close(ws, close_status_code, close_msg):
    print("### closed ###")


def on_open(ws):
    print("Opened connection")


if __name__ == "__main__":
    websocket.enableTrace(True)
    ws = websocket.WebSocketApp(addr,
                                on_open=on_open,
                                on_message=on_message,
                                on_error=on_error,
                                on_close=on_close)

    ws.run_forever(dispatcher=rel)  # Set dispatcher to automatic reconnection
    rel.signal(2, rel.abort)  # Keyboard Interrupt
    rel.dispatch()
    instruction = {
        "player_id": 1,
        "mine_id": 0,  # no mining
        "player": {
            position: {"x": 1.0, "y": 1.0, "z": 1.0},
            rotation: {"x": 1.0, "y": 1.0, "z": 1.0}
        }
    }
    ws.send(json.dumps(instruction))
    rel.signal(2, rel.abort)  # Keyboard Interrupt
    rel.dispatch()

    '''{"characters":{"2":{"player_id":2,"position":{"x_coordinate":20.332619806995766,"y_coordinate":70.26223553376647},"inventory":{}}},"ore":{"5":{"ore_id":5,"ore_type":"DRAGONHIDE","amount":2,"position":{"x_coordinate":50.251180296736806,"y_coordinate":49.79187563769072}},"4":{"ore_id":4,"ore_type":"CRYSTAL","amount":1,"position":{"x_coordinate":49.712306276201424,"y_coordinate":49.51108175573553}},"1":{"ore_id":1,"ore_type":"CRYSTAL","amount":1,"position":{"x_coordinate":50.0,"y_coordinate":50.0}},"2":{"ore_id":2,"ore_type":"DRAGONHIDE","amount":2,"position":{"x_coordinate":49.92766296156239,"y_coordinate":50.82942437941747}},"3":{"ore_id":3,"ore_type":"DRAGONHIDE","amount":1,"position":{"x_coordinate":49.37601560002236,"y_coordinate":49.490792296907514}}}}'''
