const canvas = document.getElementById('canvas');
const ctx = canvas.getContext("2d");

const config = {
    minoSize: 40,
    keys: {
        LEFT: ['ArrowLeft'], 
        RIGHT: ['ArrowRight'], 
        C_ROTATE: ['ArrowUp','x'], // ArrowUp
        CC_ROTATE: ['z'],
        ROTATE_180: ['a'],
        SOFTDROP: ['ArrowDown'], // ArrowDown
        HARDDROP: [' '], // spacebar
        HOLD: ['c', 'Shift'], // c
    }
}

// All permutations of [0,1,2,3,4,5,6]
// Modified and minified from https://stackoverflow.com/a/37580979
let permute7 = (() => {let e=[0,1,2,3,4,5,6],l=[e],r=Array(7).fill(0),t=1,u,f;for(;t<7;)r[t]<t?(u=t%2&&r[t],f=e[t],e[t]=e[u],e[u]=f,++r[t],t=1,l.push(e.slice())):(r[t]=0,++t);return l})();
// Mulberry32 PRNG from https://stackoverflow.com/a/47593316, minified
// Needed because Math.random() doesn't allow seeding
const prng = s => {r=s+=1831565813;return r=Math.imul(r^r>>>15,1|r),(((r^=r+Math.imul(r^r>>>7,61|r))^r>>>14)>>>0)/4294967296}

class TetrisSimulation {
    constructor() {

    }
}

let boardMinos = [];

// Queue
let randomSeed = Math.random() * 8198389;
let queue = permute7[Math.floor(prng(randomSeed)*5040)];
let piecesSpawned = 0;

// Hold
let isHold = false;
let holdPiece = undefined;

function createBoard() {
    // Background board
    ctx.fillStyle = "rgb(10,10,10)";
    ctx.fillRect(config.minoSize * 5, config.minoSize * 20, config.minoSize * 10, config.minoSize * 20);

    // Grid lines
    /* 
    ctx.strokeStyle = "rgb(200,200,200)";
    for (let i = 1; i < 10; i++) {
        ctx.beginPath();
        ctx.moveTo(i * 40, 0);
        ctx.lineTo(i * 40, 800);
        ctx.stroke();
    }
    for (let i = 1; i < 20; i++) {
        ctx.beginPath();
        ctx.moveTo(0, i * 40);
        ctx.lineTo(400, i * 40);
        ctx.stroke();
    }
    */
}

// --- //

// Lock
let lockInterval = null;
let lockedPosition = [];
let movesSinceSpawn = 0;

// Initialization

let T = 0;
let currentPiece = {};
setCurrentPiece();

setInterval(() => {
    T += 1;
    if (T > 80) {
        move(gravity);
        T = 0;
    }
}, 0);

// modified from https://stackoverflow.com/a/3691661
function KeyboardController(keyList) {
    let timers = {};
    document.onkeydown = event => {
        let key = event.key;
        let keyObj = keyList.find(e => e.keys.includes(key));
        if (!keyObj) return true;
        if (!(keyObj.name in timers)) {
            timers[keyObj.name] = null;
            keyObj.action();
            if (keyObj.repeat) {
                timers[keyObj.name] = setTimeout(() => {
                    timers[keyObj.name] = setInterval(keyObj.action, keyObj.repeat);
                }, keyObj.delay ?? 0);
            }
        }
    }
    document.onkeyup = event => {
        let key = event.key;
        let keyObj = keyList.find(e => e.keys.includes(key));
        if (!keyObj) return true;
        if (keyObj.name in timers) {
            if (timers[keyObj.name] !== null) {
                clearInterval(timers[keyObj.name]);
            }
            delete timers[keyObj.name];
        }
    }
}

KeyboardController([
    {
        name: 'LEFT',
        keys: config.keys.LEFT,
        repeat: 1,
        delay: 97,
        action: () => { move(moveX, -1); },
    },
    {
        name: 'RIGHT',
        keys: config.keys.RIGHT,
        repeat: 1,
        delay: 97,
        action: () => { move(moveX, 1); },
    },
    {
        name: 'SOFTDROP',
        keys: config.keys.SOFTDROP,
        repeat: 1,
        delay: 0,
        action: () => { move(moveDown, 20); }
    },
    {
        name: 'HARDDROP',
        keys: config.keys.HARDDROP,
        action: () => { move(hardDrop); }
    },
    {
        name: 'C_ROTATE',
        keys: config.keys.C_ROTATE,
        action: () => { move(rotate, 1); }
    },
    {
        name: 'CC_ROTATE',
        keys: config.keys.CC_ROTATE,
        action: () => { move(rotate, -1); }
    },
    {
        name: 'ROTATE_180',
        keys: config.keys.ROTATE_180,
        action: () => { move(rotate, 2); }
    },
    {
        name: 'HOLD',
        keys: config.keys.HOLD,
        action: () => { move(hold); }
    },
]);

createBoard();
drawShadow();
drawCurrentPiece();
drawAllMinos();

