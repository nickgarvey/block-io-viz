import { decode } from '@msgpack/msgpack';
import { animate } from "motion"

let blockDeviceSectors = 0;

function bindWebSocket(port: number) {
    const wsScheme = window.location.protocol === "https:" ? "wss" : "ws";
    const wsUrl = wsScheme + "://" + window.location.hostname + ":" + port;
    const socket = new WebSocket(wsUrl);
    socket.addEventListener('open', (event) => {
        console.log('Connection established');
    });
    socket.addEventListener('message', handleMessage);
}

fetch('/block_device.json')
    .then(response => response.json())
    .then(data => {
        const device_text = document.getElementById("device_text");
        if (device_text === null) {
            return;
        }
        device_text.innerText = data['name'];
        blockDeviceSectors = data['size_sectors'];
        bindWebSocket(data['websocket_port']);
    })
    .catch(error => {
        console.error('Error:', error);
    });



enum IOMode {
    READ = 'READ',
    WRITE = 'WRITE'
}
type BlockIOEventBuf = [
    number, // sector
    number, // nr_sectors
    [ // rwbs string each char as int
        number,
        number,
        number,
        number,
        number,
        number,
        number,
        number
    ]
];

type BlockIOEvent = {
    sector: number;
    nr_sectors: number;
    mode: IOMode;
}

function decodeBuffer(event: MessageEvent<any>): Promise<BlockIOEvent | null> {
    return event.data.arrayBuffer().then((buffer: ArrayBuffer) => {
        let event;
        try {
            event = decode(buffer) as BlockIOEventBuf;
        } catch (e) {
            console.error(e);
            return null;
        }
        const sector = event[0];
        const nr_sector = event[1];
        const rwbs = event[2];

        let mode: IOMode | null = null;

        for (const c of rwbs) {
            if (c === 87) {
                mode = IOMode.WRITE;
                break;
            } else if (c === 82) {
                mode = IOMode.READ;
                break;
            }
        }
        if (mode === null) {
            return null;
        }

        return { "sector": sector, "nr_sector": nr_sector, "mode": mode }
    });
}

let id = 0;
function genId(): string {
    return "box-" + id++;
}

const ROWS = 10;

function handleMessage(eventBuf: MessageEvent<any>) {
    let event = decodeBuffer(eventBuf).then((event) => {
        if (event === null) {
            return;
        }
        console.log(event);

        const bg = document.getElementById("render_background");
        if (bg === null) {
            return;
        }

        if (blockDeviceSectors === 0) {
            return;
        }

        let box = document.createElement('div');
        box.id = genId();
        box.classList.add('box');
        bg.appendChild(box);


        const sectorsPerRow = Math.floor(blockDeviceSectors / ROWS);
        const width = event.nr_sectors / sectorsPerRow * 100 + "%";

        const row = Math.floor(ROWS * (event.sector / blockDeviceSectors))
        box.style.width = "max(" + width + ", 1%)";
        box.style.height = (100 / ROWS) + "%";
        box.style.backgroundColor = event.mode == IOMode.READ ? "green" : "red";
        box.style.left = (event.sector % sectorsPerRow) / sectorsPerRow * 100 + "%";
        box.style.top = ((row / ROWS) * 100) + "%";

        animate(
            "#" + box.id,
            { opacity: 0, },
            { duration: 1, allowWebkitAcceleration: true }
        ).finished.then(() => {
            box.remove();
        });
    });
}