class TetrisGame extends TetrisSimulation {
    constructor(...args) {
        super(args);
    }

    createBoard() {
        // Background board
        ctx.fillStyle = "rgb(10,10,10)";
        ctx.fillRect(config.minoSize * 5, config.minoSize * 20, config.minoSize * 10, config.minoSize * 20);
    }

    updateQueue() {
        let nextPiece = super.updateQueue();
        this.drawQueue();
        return nextPiece;
    }

    clearMino(x, y) {
        ctx.fillStyle = y > 19 ? "rgb(255,255,255)" : "rgb(10,10,10)";
        ctx.fillRect(config.minoSize * (x + 5), config.minoSize * (39 - y), config.minoSize, config.minoSize);
    }

    drawMino(x, y, piece) {
        ctx.fillStyle = PIECE[piece].color;
        ctx.fillRect(config.minoSize * (x + 5), config.minoSize * (39 - y), config.minoSize, config.minoSize);
    }

    addMino(x, y, piece) {
        this.boardMinos.push({ x, y, piece });
        this.drawMino(x, y, piece);
    }

    setCurrentPiece(piece) {
        super.setCurrentPiece(piece);
        this.drawHold();
        this.drawCurrentPiece();
    }

    drawQueue() {
        ctx.fillStyle = "rgb(200,200,200)";
        ctx.fillRect(config.minoSize * 15, config.minoSize * 20, config.minoSize * 5, config.minoSize * 15);

        let queue5 = this.queue.slice(0, 5);
        for (let i = 0; i < 5; i++) {
            let piece = PIECE[queue5[i]];
            this.drawPiece(10.5 + (piece.queueOffset || 0) / 2, 16.5 - 3 * i, piece.name, 0);
        }
    }

    drawHold() {
        ctx.fillStyle = "rgb(200,200,200)";
        ctx.fillRect(0, config.minoSize * 20, config.minoSize * 5, config.minoSize * 5);

        if (this.holdPiece) {
            let piece = PIECE[this.holdPiece];
            this.drawPiece(-4.5 + (piece.queueOffset || 0) / 2, 15.5, piece.name, 0);
        }
    }

    clearCurrentPiece() {
        let pieceData = PIECE_DATA[this.currentPiece.piece];
        for (let minoPosition of pieceData[this.currentPiece.rot]) {
            this.clearMino(this.currentPiece.x + minoPosition[0], this.currentPiece.y + minoPosition[1]);
        }
    }

    drawPiece(x, y, piece, rot) {
        let pieceData = PIECE_DATA[piece];
        for (let minoPosition of pieceData[rot]) {
            this.drawMino(x + minoPosition[0], y + minoPosition[1], piece);
        }
    }

    drawCurrentPiece() {
        this.drawPiece(this.currentPiece.x, this.currentPiece.y, this.currentPiece.piece, this.currentPiece.rot);
    }

    addCurrentPiece() {
        let pieceData = PIECE_DATA[this.currentPiece.piece];
        for (let minoPosition of pieceData[this.currentPiece.rot]) {
            this.addMino(this.currentPiece.x + minoPosition[0], this.currentPiece.y + minoPosition[1], this.currentPiece.piece);
        }
    }

    clearBoard() {
        ctx.fillStyle = "rgb(10,10,10)";
        ctx.fillRect(config.minoSize * 5, config.minoSize * 20, config.minoSize * 10, config.minoSize * 20);
        ctx.fillStyle = "rgb(255,255,255)";
        ctx.fillRect(config.minoSize * 5, 0, config.minoSize * 10, config.minoSize * 20);
    }

    drawAllMinos() {
        for (let mino of this.boardMinos) {
            this.drawMino(mino.x, mino.y, mino.piece)
        }
    }

    clearLines() {
        this.clearBoard();
        super.clearLines();
        this.drawAllMinos();
    }

    drawShadow() {
        let pieceData = PIECE_DATA[this.currentPiece.piece];
        let x = this.currentPiece.x;
        let y = super.getMaximumDownDistance();
        for (let minoPosition of pieceData[this.currentPiece.rot]) {
            this.drawMino(minoPosition[0] + x, minoPosition[1] + this.currentPiece.y - y, 'SHADOW');
        }
    }

    clearShadow() {
        let pieceData = PIECE_DATA[this.currentPiece.piece];
        let x = this.currentPiece.x;
        let y = super.getMaximumDownDistance();
        for (let minoPosition of pieceData[this.currentPiece.rot]) {
            this.clearMino(minoPosition[0] + x, minoPosition[1] + this.currentPiece.y - y)
        }
    }

    move(func, ...args) {
        this.clearCurrentPiece();
        this.clearShadow();
        this.lockedPosition = { // Shallow copy
            x: this.currentPiece.x,
            y: this.currentPiece.y,
            rot: this.currentPiece.rot
        };

        let bindFunc = func.bind(this);
        bindFunc(...args);

        this.lockPiece();
        this.drawShadow();
        this.drawCurrentPiece();
    }

    KeyboardController(keyList) {
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

    hold() {
        (super.hold.bind(this))();
    }
}