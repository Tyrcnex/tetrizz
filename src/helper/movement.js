// Old method: https://www.desmos.com/calculator/ajzluy3gcd
function moveX(x) {
    if (!checkCollision(x,0)) currentPiece.x += x;
    return true;
}

function getDownDistance() {
    let piece = PIECE_DATA[currentPiece.piece.name][currentPiece.rot].map(location => ({
        x: location[0] + currentPiece.x,
        y: location[1] + currentPiece.y,
    }));
    let uniqueMinoX = [...new Set(piece.map(e => e.x))]; // this [... new Set()] thing just gets the unique values
    let filteredBoardMinos = boardMinos.filter(mino => uniqueMinoX.includes(mino.x));
    let distances = uniqueMinoX.map(x => {
        let filteredColumnPiece = piece.filter(mino => mino.x === x);
        let filteredColumnMinos = filteredBoardMinos.filter(mino => mino.x === x && mino.y <= Math.max(...filteredColumnPiece.map(e => e.y)));
        let maxColumnMino = Math.max(...filteredColumnMinos.map(e => e.y)); 
        maxColumnMino = isFinite(maxColumnMino) ? maxColumnMino : -1;
        let minPieceMino = Math.min(...filteredColumnPiece.map(e => e.y));
        return Math.abs(minPieceMino - maxColumnMino);
    });
    return Math.max(0, Math.min(...distances) - 1);
}

 function moveDown(y) { currentPiece.y -= Math.min(y, getDownDistance()); }
/*
function moveDown(y) {
    for (let i = 1; i <= y; i++) {
        if (!checkCollision(0,-1)) currentPiece.y -= 1;
        else break;
    }
    return true;
}
*/

function rotate(dir) {
    // % operator is REMAINDER, not MODULUS in js. this means that -1%4 is -1 instead of 3 as expeccted.
    // Doing this weird nest makes it work as expected.
    // https://stackoverflow.com/questions/4467539/javascript-modulo-gives-a-negative-result-for-negative-numbers
    let rotationIndex = (((currentPiece.rot + dir) % 4) + 4) % 4;
    // currentPiece.rot = rotationIndex;
    
    if (currentPiece.name === "O") currentPiece.rot = rotationIndex;
    else {
        let kickTable = {};
        if (currentPiece.piece.name === "I") kickTable = KICK_TABLE.I;
        else kickTable = KICK_TABLE.JLTSZ;

        let kickOffsets = kickTable[`r${dir > 0 ? currentPiece.rot : rotationIndex}`];
        if (!checkCollision(0,0,rotationIndex)) currentPiece.rot = rotationIndex;
        else {
            let rotationSuccess = false;
            for (let [i, offset] of kickOffsets.entries()) {
                offset = [offset[0] * Math.sign(dir), offset[1] * Math.sign(dir)];
                console.log(offset);
                if (!checkCollision(offset[0], offset[1], rotationIndex)) {
                    currentPiece.x += offset[0];
                    currentPiece.y += offset[1];
                    currentPiece.rot = rotationIndex;
                    rotationSuccess = true;
                    console.log(`Rotation ${i} used.`);
                    break;
                }
            }
            if (!rotationSuccess) console.log('Rotation unsuccessful.');
        }
    }
    return true;
}

function hardDrop() {
    moveDown(30);
    addCurrentPiece();
    clearLines();
    setCurrentPiece();
}

function gravity() {
    if (!checkCollision(0,-1)) currentPiece.y--;
}

function hold() {
    if (isHold) return;
    if (!isNaN(holdPiece)) {
        let tempHoldPiece = holdPiece;
        holdPiece = 'ZSTOIJL'.indexOf(currentPiece.piece.name);
        setCurrentPiece(tempHoldPiece);
        isHold = true;
    } else {
        holdPiece = 'ZSTOIJL'.indexOf(currentPiece.piece.name);
        setCurrentPiece();
        isHold = true;
    }
}

function move(func, ...args) {
    clearCurrentPiece();
    clearShadow();
    lockedPosition = [currentPiece.x, currentPiece.y, currentPiece.rot];
    let run = func(...args);
    if (run && !(currentPiece.x === lockedPosition[0] && currentPiece.y === lockedPosition[1] && currentPiece.rot === lockedPosition[2])) movesSinceSpawn++;
    lockPiece();
    drawShadow();
    drawCurrentPiece();
}

function lockPiece() {
    if (lockInterval && !(currentPiece.x === lockedPosition[0] && currentPiece.y === lockedPosition[1] && currentPiece.rot === lockedPosition[2]) && movesSinceSpawn < 60) { clearTimeout(lockInterval); lockInterval = null; }
    else if (checkCollision(0,-1)) {
        lockInterval = lockInterval || setTimeout(() => {
            addCurrentPiece();
            clearLines();
            setCurrentPiece();
            lockInterval = null;
        }, 500);
    }
}