msgpack = MessagePack;

const wsScheme = window.location.protocol === "https:" ? "wss" : "ws";
const wsUrl = wsScheme + "://" + window.location.hostname + ":2828";
const socket = new WebSocket(wsUrl);

socket.addEventListener('open', (event) => {
    console.log('Connection established');
})

socket.addEventListener('message', (event) => {
    event.data.arrayBuffer().then((buffer) => {
        console.log(buffer);
        const f = msgpack.decode(buffer);
        const sector = f[0];
        const nr_sector = f[1];
        const rwbs = f[2];

        let is_write = false;
        let is_read = false;

        for (const c of rwbs) {
            if (c == 87) {
                is_write = true;
                break
            } else if (c == 82) {
                is_read = true;
                break
            }
        }
        console.log(sector, nr_sector, rwbs, is_write, is_read);
    })
})