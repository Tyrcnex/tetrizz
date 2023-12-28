// first rotation to test is (0,0) for all rotations

// r0 signifies rotation 0 -> rotation 1
// to flip from rotation 1 -> rotation 0, just negate all of the values

// r3 signifies rotation 3 -> rotation 0

// https://tetris.fandom.com/wiki/SRS

const KICK_TABLE = {
    JLTSZ: {
        r0: [
            [-1, 0],
            [-1, 1],
            [ 0,-2],
            [-1,-2]
        ],
        r1: [
            [ 1, 0],
            [ 1,-1],
            [ 0, 2],
            [ 1, 2],
        ],
        r2: [
            [ 1, 0],
            [ 1, 1],
            [ 0,-2],
            [ 1,-2]
        ],
        r3: [
            [-1, 0],
            [-1,-1],
            [ 0, 2],
            [-1, 2]
        ]
    },
    I: {
        r0: [
            [-2, 0],
            [ 1, 0],
            [-2,-1],
            [ 1, 2]
        ],
        r1: [
            [-1, 0],
            [ 2, 0],
            [-1, 2],
            [ 2,-1],
        ],
        r2: [
            [ 2, 0],
            [-1, 0],
            [ 2, 1],
            [-1,-2]
        ],
        r3: [
            [ 1, 0],
            [-2, 0],
            [ 1,-2],
            [-2, 1]
        ]
    }
}