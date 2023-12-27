function clearMino(x, y) {
    ctx.fillStyle = y > 19 ? "rgb(255,255,255)" : "rgb(10,10,10)";
    ctx.fillRect(config.minoSize * (x + 5), config.minoSize * (39 - y), config.minoSize, config.minoSize);
}

function drawMino(x, y, piece) {
    ctx.fillStyle = piece?.color;
    ctx.fillRect(config.minoSize * (x + 5), config.minoSize * (39 - y), config.minoSize, config.minoSize);
}

function addMino(x, y, piece) {
    boardMinos.push({ x, y, piece });
    drawMino(x, y, piece);
}

// Returns the next piece in the queue
function updateQueue() {
    piecesSpawned++;
    let nextPiece = queue.shift();
    let nextIndex = permute7[Math.floor(prng(randomSeed+Math.floor((piecesSpawned-1)/7)+1)*5040)][piecesSpawned % 7];
    queue.push(nextIndex);
    drawQueue();
    return nextPiece;
}

function setCurrentPiece(piece) {
    isHold = false;
    let nextPiece = piece ?? updateQueue();
    drawHold();

    currentPiece = {
        x: 3,
        y: 19,
        rot: 0,
        piece: PIECE['ZSTOIJL'[nextPiece]]
    };
    movesSinceSpawn = 0;
    if (checkCollision()) { currentPiece.y++; }
    if (checkCollision()) { clearEverything(); }
}

function drawQueue() {
    ctx.fillStyle = "rgb(200,200,200)";
    ctx.fillRect(config.minoSize * 15,config.minoSize * 20,config.minoSize * 5,config.minoSize * 15);

    let queue5 = queue.slice(0,5);
    for (let i = 0; i < 5; i++) {
        let piece = PIECE['ZSTOIJL'[queue5[i]]];
        drawPiece(10.5+(piece.queueOffset || 0)/2, 16.5-3*i, piece, 0);
    }
}

function drawHold() {
    ctx.fillStyle = "rgb(200,200,200)";
    ctx.fillRect(0,config.minoSize * 20,config.minoSize * 5,config.minoSize * 5);

    if (!isNaN(holdPiece)) {
        let piece = PIECE['ZSTOIJL'[holdPiece]]
        drawPiece(-4.5+(piece.queueOffset || 0)/2, 15.5, piece, 0);
    }
}

function clearCurrentPiece() {
    let pieceData = PIECE_DATA[currentPiece.piece.name];
    for (let minoPosition of pieceData[currentPiece.rot]) {
        clearMino(currentPiece.x + minoPosition[0], currentPiece.y + minoPosition[1]);
    }
}

function drawPiece(x, y, piece, rot) {
    let pieceData = PIECE_DATA[piece.name];
    for (let minoPosition of pieceData[rot]) {
        drawMino(x + minoPosition[0], y + minoPosition[1], piece);
    }
}

function drawCurrentPiece() {
    drawPiece(currentPiece.x, currentPiece.y, currentPiece.piece, currentPiece.rot);
}

function addCurrentPiece() {
    let pieceData = PIECE_DATA[currentPiece.piece.name];
    for (let minoPosition of pieceData[currentPiece.rot]) {
        addMino(currentPiece.x + minoPosition[0], currentPiece.y + minoPosition[1], currentPiece.piece);
    }
}

function clearBoard() {
    ctx.fillStyle = "rgb(10,10,10)";
    ctx.fillRect(config.minoSize * 5, config.minoSize * 20, config.minoSize * 10, config.minoSize * 20);
    ctx.fillStyle = "rgb(255,255,255)";
    ctx.fillRect(config.minoSize * 5, 0, config.minoSize * 10, config.minoSize * 20);
}

function drawAllMinos() {
    for (let mino of boardMinos) {
        drawMino(mino.x, mino.y, PIECE[mino.piece.name])
    }
}

function clearLines() {
    clearBoard();
    let newMinos = [];
    let clearedLines = 0;
    let allMinoY = [...new Set(boardMinos.map(e => e.y))]; // this [... new Set()] thing just gets the unique values
    for (let y of allMinoY.sort((a,b)=>a-b)) {
        let minosInColumn = boardMinos.filter(mino => mino.y === y);
        if (minosInColumn.length === 10) {
            clearedLines++;
        } else newMinos.push(minosInColumn.map(mino => {
            mino.y -= clearedLines;
            return mino;
        }));
    }
    boardMinos = newMinos.flat();
    drawAllMinos();
}

function drawShadow() {
    let pieceData = PIECE_DATA[currentPiece.piece.name];
    let x = currentPiece.x;
    let y = getDownDistance();
    for (let minoPosition of pieceData[currentPiece.rot]) {
        drawMino(minoPosition[0] + x, minoPosition[1] + currentPiece.y - y, PIECE.SHADOW);
    }
}

function clearShadow() {
    let pieceData = PIECE_DATA[currentPiece.piece.name];
    let x = currentPiece.x;
    let y = getDownDistance();
    for (let minoPosition of pieceData[currentPiece.rot]) {
        clearMino(minoPosition[0] + x, minoPosition[1] + currentPiece.y - y)
    }
}

function clearEverything() {
    clearBoard();
    boardMinos = [];
    randomSeed = Math.random() * 8198389;
    queue = permute7[Math.floor(prng(randomSeed)*5040)];
    piecesSpawned = 0;
    setCurrentPiece();
    drawCurrentPiece();
}