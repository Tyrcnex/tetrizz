import * as child_process from "child_process";
import express from "express";
import cors from "cors";

const app = express();
const port = 3000;

app.use(express.json());
app.use(cors());

app.post('/', (req, res) => {
    const data = req.body; // Already parsed by express.json()
    requestMove(data.board, data.hold, data.queue).then(x => res.send(x));
})

app.listen(port, () => {
    console.log(`Example app listening on port ${port}`)
})




const botEngine = {
    engine: child_process.spawn("../target/release/keygen"),
    keyInfo: {}
};

botEngine.engine.stdout.on("data", data => {
    data = JSON.parse(data.toString().trim());
    botEngine.keyInfo.allKeys = [];
    for (const key of data) {
        botEngine.keyInfo.allKeys.push({ type: "keydown", key, sleep: key == "DASLeft" || key == "DASRight" ? 0.7 : key == "Softdrop" ? 1 : 0.2 });
        botEngine.keyInfo.allKeys.push({ type: "keyup", key, sleep: 0.2 });
    }
    botEngine.keyInfo.length = data.keys.length;
    // botEngine.keyInfo.desiredLocation = {
    //     piece: data.desired_location.piece,
    //     x: data.desired_location.x,
    //     y: data.desired_location.y,
    //     rotation: { "North": 0, "East": 1, "South": 2, "West": 3 }[data.desired_location.rotation]
    // };
});

botEngine.engine.stderr.on("data", data => {
    console.error(data.toString());
    botEngine.keyInfo.error = data.toString();
});

async function requestMove(board, hold, queue) {
    botEngine.keyInfo = {};

    let board_new = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    for (let row = 0; row < 20; row++) {
        for (let col = 0; col < 10; col++) {
            board_new[col] += +!!board[row][col] * (1 << row);
        }
    }

    let depth = queue.length;

    let input = {
        game: {
            board: { cols: board_new },
            hold,
            b2b: 67,
            combo: 0,
            incoming_garbage: 0
        },
        queue,
        beam_width: Math.floor(100000 / (depth * 4)),
        beam_depth: depth,
        human: true
    };

    console.log("\n\n\n\n\n\n\n", input);

    botEngine.engine.stdin.write(JSON.stringify(input) + "\n");

    return new Promise(res => setInterval(_ => {
        if (Object.keys(botEngine.keyInfo).length == 0) return;
        res(botEngine.keyInfo);
    }, 0));
}

function what_the_fuck_is_this() {
window.sleep = ms => new Promise(r => setTimeout(r, ms));
function makeMove() {
    let board = structuredClone(GAME_OBJECT.matrix);
    board.reverse();
    fetch("http://localhost:3000/", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ board: board.map(r => r.map(c => +!!c)), hold: "IOTLJSZ"[GAME_OBJECT.blockInHold?.id], queue: [GAME_OBJECT.activeBlock, ...GAME_OBJECT.queue].map(p => "IOTLJSZ"[p.id]) }),
    }).then(x => x.json()).then(async x => {
        const data = {
            "Softdrop": 40,
            "TapLeft": 37,
            "TapRight": 39,
            "DASLeft": 37,
            "DASRight": 39,
            "RotateCW": 38,
            "RotateCCW": 90,
            "Rotate180": 65,
            "Harddrop": 32,
            "Hold": 67
        };
        for await (const key of x.allKeys) {
            let func = (key.type == "keydown" ? GAME_OBJECT.keyInput2 : GAME_OBJECT.keyInput3).bind(GAME_OBJECT);
            let obj = { keyCode: data[key.key], timestamp: performance.now(), shiftKey: false, preventDefault: _ => { }, stopPropagation: _ => { }, stopImmediatePropagation: _ => { } };
            func(obj);
            await window.sleep(key.sleep * 16.67);
        }
        await window.sleep(20);
        makeMove();
    });
}
setTimeout(makeMove, 2000);
}