// True for collision, false for no collision

// x and y are optional. These variables are just to offset the piece for checking if needed.
// For example, checkCollision(0,-1) checks if there is anything UNDERNEATH the piece
function checkCollision(x,y,rot) {
    x ??= 0;
    y ??= 0;
    rot ??= currentPiece.rot;
    let piece = PIECE_DATA[currentPiece.piece.name][rot].map(location => ({
        x: location[0] + currentPiece.x + x,
        y: location[1] + currentPiece.y + y,
    }));
    if (piece.some(location => location.x < 0 || location.x > 9 || location.y < 0)) return true;
    return boardMinos.some(mino => piece.some(location => location.x === mino.x && location.y === mino.y));
}