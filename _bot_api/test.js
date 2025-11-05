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
    botEngine.keyInfo.allKeys = data.keys;
    botEngine.keyInfo.length = data.keys.length;
    botEngine.keyInfo.desiredLocation = {
        piece: data.desired_location.piece,
        x: data.desired_location.x,
        y: data.desired_location.y,
        rotation: { "Up": 0, "Right": 1, "Down": 2, "Left": 3 }[data.desired_location.rotation]
    };
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

    let input = {
        game: {
            board: { cols: board_new },
            hold,
            b2b: 0,
            b2b_deficit: 0
        },
        queue,
        beam_width: 100000,
        beam_depth: queue.length,
        impending: 0
    };

    console.log(input);

    botEngine.engine.stdin.write(JSON.stringify(input) + "\n");

    return new Promise(res => setInterval(_ => {
        if (Object.keys(botEngine.keyInfo).length == 0) return;
        res(botEngine.keyInfo);
    }, 0));
}

function what_the_fuck_is_this() {
    const sleep = ms => new Promise(r => setTimeout(r, ms));
    setInterval(_ => {
        let board = structuredClone(GAME_OBJECT.matrix);
        board.reverse();
        fetch("http://localhost:3000/", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ board: board.map(r => r.map(c => +!!c)), hold: "IOTLJSZ"[GAME_OBJECT.blockInHold.id], queue: [GAME_OBJECT.activeBlock, ...GAME_OBJECT.queue].map(p => "IOTLJSZ"[p.id]) }),
        }).then(x => x.json()).then(async x => {
            const data = {
                "softDrop": 40,
                "moveLeft": 37,
                "moveRight": 39,
                "rotateCW": 38,
                "rotateCCW": 90,
                "rotate180": 65,
                "hardDrop": 32,
                "hold": 67
            };
            console.log([...x.allKeys.slice(0,-2), "hardDrop"])
            for await (const key of [...x.allKeys.slice(0,-2), "hardDrop"]) {
                GAME_OBJECT.keyInput2({ keyCode: data[key], timestamp: performance.now(), shiftKey: false, preventDefault: _ => { }, stopPropagation: _ => { }, stopImmediatePropagation: _ => { } });
                if (key == "softDrop" || key == "hardDrop") await sleep(10);
                GAME_OBJECT.keyInput3({ keyCode: data[key], timestamp: performance.now(), shiftKey: false, preventDefault: _ => { }, stopPropagation: _ => { }, stopImmediatePropagation: _ => { } });
                await sleep(100);
            }
        });
    }, 2000);
}