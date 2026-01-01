import * as child_process from "child_process";
import express from "express";
import cors from "cors";

const app = express();
const port = 3000;

app.use(express.json());
app.use(cors());

app.post('/', (req, res) => {
    const data = req.body;
    requestMove(data.board, data.hold, data.queue, data.settings).then(x => res.send(x));
})

app.listen(port, () => {
    console.log(`Example app listening on port ${port}`)
})




const botEngine = {
    engine: child_process.spawn("../target/release/keygen"),
    keyInfo: {}
};

botEngine.engine.stdout.on("data", data => {
    botEngine.keyInfo.allKeys = JSON.parse(data.toString().trim());
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

async function requestMove(board, hold, queue, settings) {
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
            b2b: 7,
            combo: -1,
            incoming_garbage: 0
        },
        queue,
        beam_width: Math.floor(100000 / (depth * settings.pps)),
        beam_depth: depth,
        human: settings.human || true,
        debug: settings.debug || false
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
window.done = false;
window.settings = { pps: 2, human: false, das: 20 };
async function makeMove() {
    if (window.done) { return; }
    if (GAME_OBJECT.gameEnded) {
        await window.sleep(100);
        // GAME_OBJECT.keyInput2({ keyCode: 82, timestamp: performance.now(), shiftKey: false, preventDefault: _ => { }, stopPropagation: _ => { }, stopImmediatePropagation: _ => { } });
        // await window.sleep(20);
        return await makeMove();
    }
    let board = structuredClone(GAME_OBJECT.matrix);
    const BLOCKS = ["I5","V5","T5","U5","W5","X5","J5","L5","S5","Z5","TL","TJ","OZ","OS","TS","TZ","LL","JJ"];
    board.reverse();
    fetch("http://localhost:3000/", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
            board: board.map(r => r.map(c => +!!c)),
            hold: BLOCKS[GAME_OBJECT.blockInHold?.id],
            queue: [GAME_OBJECT.activeBlock, ...GAME_OBJECT.queue].map(p => BLOCKS[p.id]),
            settings
        }),
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
        let allKeys = [];
        for (const key of x.allKeys) {
            allKeys.push({ type: "keydown", key, sleep: key == "DASLeft" || key == "DASRight" ? settings.das * 1.1 : key == "Softdrop" ? 18 : settings.das * 0.5 });
            allKeys.push({ type: "keyup", key, sleep: settings.das * 0.5 });
        }
        for await (const key of x.allKeys) {
            let func = (key.type == "keydown" ? GAME_OBJECT.keyInput2 : GAME_OBJECT.keyInput3).bind(GAME_OBJECT);
            let obj = { keyCode: data[key.key], timestamp: performance.now(), shiftKey: false, preventDefault: _ => { }, stopPropagation: _ => { }, stopImmediatePropagation: _ => { } };
            func(obj);
            await window.sleep(key.sleep);
        }
        makeMove();
    });
}
setTimeout(makeMove, 2000);
}