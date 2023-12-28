/**
 * Simulates a Tetris game, with all the moving parts, without actually drawing anything.
 */
class TetrisSimulation {
    /**
     * Constructor for each simulation.
     * @constructor
     * Board
     * @param {Array} [boardMinos = []]
     * Queue - The queue variable will be generated with these two variables
     * @param {Number} randomSeed
     * @param {Number} [piecesSpawned = 0]
     * Hold
     * @param {Boolean} [isHold = false]
     * @param {String} holdPiece
     * Lock
     * @param {Number} lockInterval - This is actually a setTimeout, which returns a number
     * Initialization
     * @param {Object} [currentPiece = []]
     * @param {Number} currentPiece.x - This is from 0 to 9.
     * @param {Number} currentPiece.y - This is greater than 0, although it shouldn't go over 23.
     * @param {Number} currentPiece.rot - There are only four possible rotations: 0, 1, 2, and 3.
     * @param {Object} currentPiece.piece - This is the piece data. I will change this later to just have a string, and access that data when I run the code.
     */
    constructor(
        boardMinos = [],
        randomSeed,
        piecesSpawned = 0,
        isHold = false,
        holdPiece,
        lockInterval,
        currentPiece = {},
    ) {
        this.boardMinos = boardMinos;

        this.randomSeed = randomSeed || Math.random() * 8198389;
        this.queue = permute7[Math.floor(prng(this.randomSeed) * 5040)];
        this.piecesSpawned = piecesSpawned;

        this.isHold = isHold;
        this.holdPiece = holdPiece;

        this.lockInterval = lockInterval;
        this.lockPosition = { x: -1, y: -1, rot: -1 }

        this.currentPiece = { // This is for a copy of the object, not the reference
            x: currentPiece.x,
            y: currentPiece.y,
            rot: currentPiece.rot,
            piece: currentPiece.piece
        }

        this.endGame = false;
    }

    /**
     * HELPER FUNCTION
     * @returns {Array} Given the offset of the piece (currentPiece.x and currentPiece.y), and the mino positions on that piece (for the S piece, for example, it should be something like [(0,0),(1,0),(1,1),(2,2)]), I just generate all of those positions plus the offset.
     */
    getAllPieceMinos() {
        return PIECE_DATA[this.currentPiece.piece][this.currentPiece.rot].map(location => ({
            x: location[0] + this.currentPiece.x,
            y: location[1] + this.currentPiece.y,
        }));
    }

    checkCollision(x = 0, y = 0, rot = this.currentPiece.rot) {
        let piece = PIECE_DATA[this.currentPiece.piece][rot].map(location => ({
            x: location[0] + this.currentPiece.x + x,
            y: location[1] + this.currentPiece.y + y,
        }));
        if (piece.some(location => location.x < 0 || location.x > 9 || location.y < 0)) return true;
        return this.boardMinos.some(mino => piece.some(location => location.x === mino.x && location.y === mino.y));
    }

    /** 
     * Moves the piece by x units. 
     * This doesn't actually do any fancy math collisions, because x is usually just -1 or 1.
     * @param {Number} x - Positive for moving right, negative for moving left. This is usually just -1 or 1. If it's greater than that, it might glitch through walls.
     */
    moveX(x) {
        if (!this.checkCollision(x, 0)) this.currentPiece.x += x;
    }

    /**
     * HELPER FUNCTION
     * Takes the current piece and figures out the maximum distance it can go down before it hits a board mino.
     * Demonstration of how this works: https://www.desmos.com/calculator/ngzfoeinnz
     * @returns {Number}
     */
    getMaximumDownDistance() {
        let piece = this.getAllPieceMinos();
        let uniqueMinoX = [...new Set(piece.map(e => e.x))]; // this [... new Set()] thing just gets the unique values
        let filteredBoardMinos = this.boardMinos.filter(mino => uniqueMinoX.includes(mino.x));
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

    /**
     * Moves the piece down by y units.
     * This uses the getDownDistance function because y is very large for SDF(soft drop factor)=infinity or for hard drops.
     * @param {Number} y - This should be greater than 0. If soft dropping, y should be 1 (unless SDF(soft drop factor)=infinity), so the function optimizes. If hard dropping, y should be a pretty large value.
     */
    moveDown(y) {
        if (y === 1) { // Optimization for y=1
            if (!this.checkCollision(0, -1)) this.currentPiece.y -= 1;
        } else {
            this.currentPiece.y -= Math.min(y, this.getMaximumDownDistance());
        }
    }

    /**
     * Rotates the piece to a direction dir.
     * Implements SRS (Super Rotation System), so it has super wallkicks. https://tetris.fandom.com/wiki/SRS
     * The SRS kick table is in the variable named KICK_TABLE
     * @param {Number} dir - Should be either 0, 1, 2, or 3.
     */
    rotate(dir) {
        // % operator is REMAINDER, not MODULUS in js. this means that -1%4 is -1 instead of 3 as expeccted.
        // Doing this weird nest makes it work as expected.
        // https://stackoverflow.com/questions/4467539/javascript-modulo-gives-a-negative-result-for-negative-numbers
        let rotationIndex = (((this.currentPiece.rot + dir) % 4) + 4) % 4;
        // currentPiece.rot = rotationIndex;

        if (this.currentPiece.piece === "O") this.currentPiece.rot = rotationIndex;
        else {
            let kickTable = {};
            if (this.currentPiece.piece === "I") kickTable = KICK_TABLE.I;
            else kickTable = KICK_TABLE.JLTSZ;

            let kickOffsets = kickTable[`r${dir > 0 ? this.currentPiece.rot : rotationIndex}`];
            if (!this.checkCollision(0, 0, rotationIndex)) this.currentPiece.rot = rotationIndex;
            else {
                let rotationSuccess = false;
                for (let [i, offset] of kickOffsets.entries()) {
                    offset = [offset[0] * Math.sign(dir), offset[1] * Math.sign(dir)];
                    console.log(`Testing offset ${i}: [${offset}]`);
                    if (!this.checkCollision(offset[0], offset[1], rotationIndex)) {
                        this.currentPiece.x += offset[0];
                        this.currentPiece.y += offset[1];
                        this.currentPiece.rot = rotationIndex;
                        rotationSuccess = true;
                        console.log(`Rotation ${i} used.`);
                        break;
                    }
                }
                if (!rotationSuccess) console.log('Rotation unsuccessful.');
            }
        }
    }

    hardDrop() {
        this.moveDown(30);
        this.addCurrentPiece();
        this.clearLines();
        this.setCurrentPiece();
    }

    hold() {
        if (this.isHold) return;
        if (this.holdPiece) {
            let tempHoldPiece = this.holdPiece;
            this.holdPiece = this.currentPiece.piece;
            this.setCurrentPiece(tempHoldPiece);
            this.isHold = true;
        } else {
            this.holdPiece = this.currentPiece.piece;
            this.setCurrentPiece();
            this.isHold = true;
        }
    }

    updateQueue() {
        this.piecesSpawned++;
        let nextPiece = this.queue.shift();

        let nextIndex = permute7[
            Math.floor(
                prng(
                    this.randomSeed +
                    Math.floor((this.piecesSpawned - 1) / 7) + 1
                ) * 5040
            )
        ][this.piecesSpawned % 7];

        this.queue.push(nextIndex);
        return nextPiece;
    }

    // piece is optional (put in JSDoc later)
    setCurrentPiece(piece = this.updateQueue()) {
        this.isHold = false;

        this.currentPiece = {
            x: 3,
            y: 19,
            rot: 0,
            piece: piece
        };
        this.movesSinceSpawn = 0;
        if (this.checkCollision()) { this.currentPiece.y++; }
        if (this.checkCollision()) { this.endGame = true; }
    }

    lockPiece() {
        if (
            this.lockInterval &&
            !(
                this.currentPiece.x === this.lockedPosition.x &&
                this.currentPiece.y === this.lockedPosition.y &&
                this.currentPiece.rot === this.lockedPosition.rot
            )
        ) {
            clearTimeout(this.lockInterval);
            this.lockInterval = null;
        }
        else if (this.checkCollision(0, -1)) {
            this.lockInterval = this.lockInterval || setTimeout(() => {
                this.addCurrentPiece();
                this.clearLines();
                this.setCurrentPiece();
                this.lockInterval = null;
            }, 500);
        }
    }

    addCurrentPiece() {
        this.getAllPieceMinos().forEach(mino => {
            this.boardMinos.push(mino);
        });
    }

    gravity() {
        this.moveDown(1);
    }

    clearLines() {
        let newMinos = [];
        let clearedLines = 0;
        let allMinoY = [...new Set(this.boardMinos.map(e => e.y))]; // this [... new Set()] thing just gets the unique values
        for (let y of allMinoY.sort((a, b) => a - b)) {
            let minosInColumn = this.boardMinos.filter(mino => mino.y === y);
            if (minosInColumn.length === 10) {
                clearedLines++;
            } else newMinos.push(minosInColumn.map(mino => {
                mino.y -= clearedLines;
                return mino;
            }));
        }
        this.boardMinos = newMinos.flat();
    }
}