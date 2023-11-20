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
type BlockIOEvent = [
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


socket.addEventListener('message', (event) => {
    event.data.arrayBuffer().then((buffer: ArrayBuffer) => {
        const event = decode(buffer) as BlockIOEvent;
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
        console.log(sector, nr_sector, mode);
    })
})