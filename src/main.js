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
let permute7 = (() => {let e='ZSTOIJL'.split(''),l=[e],r=Array(7).fill(0),t=1,u,f;for(;t<7;)r[t]<t?(u=t%2&&r[t],f=e[t],e[t]=e[u],e[u]=f,++r[t],t=1,l.push(e.slice())):(r[t]=0,++t);return l})();
// Mulberry32 PRNG from https://stackoverflow.com/a/47593316, minified
// Needed because Math.random() doesn't allow seeding
const prng = s => {r=s+=1831565813;return r=Math.imul(r^r>>>15,1|r),(((r^=r+Math.imul(r^r>>>7,61|r))^r>>>14)>>>0)/4294967296}

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

let game = new TetrisGame();
game.KeyboardController([
    {
        name: 'LEFT',
        keys: config.keys.LEFT,
        repeat: 1,
        delay: 97,
        action: () => { game.move(game.moveX, -1); },
    },
    {
        name: 'RIGHT',
        keys: config.keys.RIGHT,
        repeat: 1,
        delay: 97,
        action: () => { game.move(game.moveX, 1); },
    },
    {
        name: 'SOFTDROP',
        keys: config.keys.SOFTDROP,
        repeat: 1,
        delay: 0,
        action: () => { game.move(game.moveDown, 20); }
    },
    {
        name: 'HARDDROP',
        keys: config.keys.HARDDROP,
        action: () => { game.move(game.hardDrop); }
    },
    {
        name: 'C_ROTATE',
        keys: config.keys.C_ROTATE,
        action: () => { game.move(game.rotate, 1); }
    },
    {
        name: 'CC_ROTATE',
        keys: config.keys.CC_ROTATE,
        action: () => { game.move(game.rotate, -1); }
    },
    {
        name: 'ROTATE_180',
        keys: config.keys.ROTATE_180,
        action: () => { game.move(game.rotate, 2); }
    },
    {
        name: 'HOLD',
        keys: config.keys.HOLD,
        action: () => { game.move(game.hold); }
    },
]);

let T = 0;
createBoard();
game.setCurrentPiece();
game.drawCurrentPiece();
let gameLoopInterval = setInterval(() => {
    T += 1;
    if (T > 80/(game.piecesSpawned * 0.2 + 1)) {
        game.move(game.gravity);
        T = 0;
    }
    if (game.endGame) { 
        clearInterval(gameLoopInterval);
    }
}, 0);