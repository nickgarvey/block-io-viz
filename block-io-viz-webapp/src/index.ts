import { decode } from '@msgpack/msgpack';
import { animate } from "motion"

const wsScheme = window.location.protocol === "https:" ? "wss" : "ws";
const wsUrl = wsScheme + "://" + window.location.hostname + ":2828";
const socket = new WebSocket(wsUrl);

socket.addEventListener('open', (event) => {
    console.log('Connection established');
})

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

socket.addEventListener('message', (eventBuf) => {
    let event = decodeBuffer(eventBuf).then((event) => {
        if (event === null) {
            return;
        }
        console.log(event);

        const bg = document.getElementById("render_background");
        if (bg === null) {
            return;
        }

        let box = document.createElement('div');
        box.id = genId();
        box.classList.add('box');
        bg.appendChild(box);

        const BLOCK_DEV_SECTORS = Math.floor(68719476736 / 512);
        const ROWS = 10;

        const sectorsPerRow = Math.floor(BLOCK_DEV_SECTORS / ROWS);
        const width = event.nr_sectors / sectorsPerRow * 100 + "%";

        const row = Math.floor(ROWS * (event.sector / BLOCK_DEV_SECTORS))
        box.style.width = "max(" + width + ", 1%)";
        box.style.height = (100 / ROWS) + "%";
        box.style.backgroundColor = event.mode == IOMode.READ ? "green" : "red";
        box.style.left = (event.sector % sectorsPerRow) / sectorsPerRow * 100 + "%";
        box.style.top = ((row / ROWS) * 100) + "%";


        animate(
            "#" + box.id,
            {
                opacity: 0,
            },
            { duration: 1, allowWebkitAcceleration: true }
        ).finished.then(() => {
            box.remove();
        });
    });
});